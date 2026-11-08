use std::ffi::c_void;
use std::path::Path;

use whisper_rs::{
    get_lang_id, get_lang_str, FullParams, SamplingStrategy, WhisperContext,
    WhisperContextParameters,
};

unsafe extern "C" fn silence_log(_level: u32, _text: *const i8, _user_data: *mut c_void) {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Txt,
    Srt,
    Vtt,
    Json,
}

impl Format {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "txt" | "text" => Ok(Self::Txt),
            "srt" => Ok(Self::Srt),
            "vtt" => Ok(Self::Vtt),
            "json" => Ok(Self::Json),
            other => Err(format!(
                "unknown format '{other}' (expected txt, srt, vtt or json)"
            )),
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Srt => "srt",
            Self::Vtt => "vtt",
            Self::Json => "json",
        }
    }
}

/// One transcribed segment. Timestamps are in milliseconds.
#[derive(Debug, Clone)]
pub struct Segment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

/// A complete transcription result with metadata.
#[derive(Debug, Clone)]
pub struct Transcript {
    /// Detected or requested language code (e.g. `en`), when known.
    pub language: Option<String>,
    /// Audio duration in milliseconds.
    pub duration_ms: i64,
    pub segments: Vec<Segment>,
}

/// Validates a language argument: `"auto"` or a whisper language code.
pub fn validate_language(language: &str) -> Result<(), String> {
    if language == "auto" || get_lang_id(language).is_some() {
        Ok(())
    } else {
        Err(format!(
            "unknown language '{language}'; use 'auto' or a code like 'en', 'vi', 'ja'"
        ))
    }
}

/// Transcribes 16 kHz mono WAV audio at `audio` using the whisper model at
/// `model`. `language` is a whisper language code or `"auto"`.
/// Progress is reported to stderr as a percentage.
pub fn transcribe(
    audio: &Path,
    model: &Path,
    language: &str,
    threads: usize,
) -> Result<Transcript, String> {
    let samples = read_wav_f32(audio)?;
    let duration_ms = (samples.len() as i64) / 16;

    // whisper.cpp logs to stderr by default; silence it.
    unsafe {
        whisper_rs::set_log_callback(
            whisper_rs::WhisperLogCallback::Some(silence_log),
            std::ptr::null_mut(),
        );
    }

    let ctx = WhisperContext::new_with_params(model, WhisperContextParameters::default())
        .map_err(|e| format!("cannot load model {}: {e}", model.display()))?;
    let mut state = ctx
        .create_state()
        .map_err(|e| format!("cannot create whisper state: {e}"))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 5 });
    params.set_n_threads(threads as i32);
    params.set_print_progress(false);
    params.set_progress_callback_safe(|progress| {
        eprint!("\rtranscribing... {progress}%   ");
    });
    params.set_print_realtime(false);
    params.set_print_special(false);
    params.set_print_timestamps(false);
    if language == "auto" {
        params.set_language(None);
    } else {
        params.set_language(Some(language));
    }

    state
        .full(params, &samples)
        .map_err(|e| format!("whisper transcription failed: {e}"))?;

    let language = get_lang_str(state.full_lang_id_from_state()).map(str::to_string);
    let mut segments = Vec::new();
    for seg in state.as_iter() {
        let text = seg
            .to_str()
            .map_err(|e| format!("cannot read segment text: {e}"))?
            .trim()
            .to_string();
        if text.is_empty() {
            continue;
        }
        segments.push(Segment {
            start_ms: seg.start_timestamp() * 10,
            end_ms: seg.end_timestamp() * 10,
            text,
        });
    }
    eprintln!();
    Ok(Transcript {
        language,
        duration_ms,
        segments,
    })
}

/// Renders a transcript in the requested output format.
pub fn render(format: Format, transcript: &Transcript) -> String {
    let segments = &transcript.segments;
    match format {
        Format::Txt => segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        Format::Srt => {
            let mut out = String::new();
            for (i, s) in segments.iter().enumerate() {
                out.push_str(&format!(
                    "{}\n{} --> {}\n{}\n\n",
                    i + 1,
                    srt_ts(s.start_ms),
                    srt_ts(s.end_ms),
                    s.text
                ));
            }
            out
        }
        Format::Vtt => {
            let mut out = String::from("WEBVTT\n\n");
            for s in segments {
                out.push_str(&format!(
                    "{} --> {}\n{}\n\n",
                    vtt_ts(s.start_ms),
                    vtt_ts(s.end_ms),
                    s.text
                ));
            }
            out
        }
        Format::Json => {
            let mut out = String::from("{\"language\":");
            match &transcript.language {
                Some(l) => out.push_str(&json_escape(l)),
                None => out.push_str("null"),
            }
            out.push_str(&format!(
                ",\"duration_ms\":{},\"segments\":[",
                transcript.duration_ms
            ));
            for (i, s) in segments.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!(
                    "{{\"start\":{},\"end\":{},\"text\":{}}}",
                    s.start_ms,
                    s.end_ms,
                    json_escape(&s.text)
                ));
            }
            out.push_str("]}\n");
            out
        }
    }
}

fn srt_ts(ms: i64) -> String {
    let (h, m, s, milli) = split_ts(ms);
    format!("{h:02}:{m:02}:{s:02},{milli:03}")
}

fn vtt_ts(ms: i64) -> String {
    let (h, m, s, milli) = split_ts(ms);
    format!("{h:02}:{m:02}:{s:02}.{milli:03}")
}

fn split_ts(ms: i64) -> (i64, i64, i64, i64) {
    let ms = ms.max(0);
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1000;
    let milli = ms % 1000;
    (h, m, s, milli)
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Reads a PCM 16-bit little-endian mono WAV file and returns f32 samples
/// in [-1.0, 1.0] at the file's own sample rate. The caller is expected to
/// have produced 16 kHz audio via `ffmpeg::extract_audio`.
fn read_wav_f32(path: &Path) -> Result<Vec<f32>, String> {
    let data = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err(format!("{} is not a WAV file", path.display()));
    }

    // Walk chunks to find "fmt " and "data".
    let mut fmt: Option<(u16, u16, u32)> = None; // (channels, bits, sample_rate)
    let mut pcm: Option<&[u8]> = None;
    let mut pos = 12usize;
    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let body = pos + 8;
        if body + size > data.len() {
            break;
        }
        match id {
            b"fmt " if size >= 16 => {
                let audio_format = u16::from_le_bytes(data[body..body + 2].try_into().unwrap());
                let channels = u16::from_le_bytes(data[body + 2..body + 4].try_into().unwrap());
                let rate = u32::from_le_bytes(data[body + 4..body + 8].try_into().unwrap());
                let bits = u16::from_le_bytes(data[body + 14..body + 16].try_into().unwrap());
                if audio_format != 1 {
                    return Err(format!(
                        "{}: unsupported WAV encoding {audio_format}",
                        path.display()
                    ));
                }
                fmt = Some((channels, bits, rate));
            }
            b"data" => pcm = Some(&data[body..body + size]),
            _ => {}
        }
        pos = body + size + (size & 1);
    }

    let (channels, bits, rate) =
        fmt.ok_or_else(|| format!("{}: missing fmt chunk", path.display()))?;
    if rate != 16_000 {
        return Err(format!(
            "{}: expected 16 kHz audio, got {rate} Hz",
            path.display()
        ));
    }
    if bits != 16 {
        return Err(format!(
            "{}: expected 16-bit PCM, got {bits}-bit",
            path.display()
        ));
    }
    let pcm = pcm.ok_or_else(|| format!("{}: missing data chunk", path.display()))?;
    let channels = channels as usize;
    if channels == 0 {
        return Err(format!("{}: zero channels", path.display()));
    }

    // Downmix to mono by averaging channels, then scale i16 to f32.
    let frames = pcm.len() / 2 / channels;
    let mut out = Vec::with_capacity(frames);
    for i in 0..frames {
        let mut sum = 0i64;
        for ch in 0..channels {
            let off = (i * channels + ch) * 2;
            sum += i16::from_le_bytes([pcm[off], pcm[off + 1]]) as i64;
        }
        out.push((sum / channels as i64) as f32 / 32768.0);
    }
    Ok(out)
}
