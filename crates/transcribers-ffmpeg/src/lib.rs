use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};

/// Returns true if `input` contains at least one audio stream, probed via
/// `ffprobe`. Returns `Ok(None)` when ffprobe is unavailable so callers can
/// skip the check rather than fail.
pub fn has_audio_stream(input: &Path) -> Result<Option<bool>, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "csv=p=0",
        ])
        .arg(input)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let found = String::from_utf8_lossy(&out.stdout)
                .lines()
                .any(|l| l.trim() == "audio");
            Ok(Some(found))
        }
        Ok(_) => Ok(None),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("failed to run ffprobe: {e}")),
    }
}

/// Options for audio extraction.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExtractOptions {
    /// Start offset in seconds; extraction begins there.
    pub start_s: Option<f64>,
    /// Maximum duration in seconds; `None` extracts to the end.
    pub duration_s: Option<f64>,
}

/// Extracts the audio track of `input` into a 16 kHz mono WAV at `wav`.
pub fn extract_audio(input: &Path, wav: &Path) -> Result<(), String> {
    extract_audio_with(input, wav, &ExtractOptions::default())
}

/// Extracts audio like [`extract_audio`], honouring `opts` for clip ranges.
pub fn extract_audio_with(input: &Path, wav: &Path, opts: &ExtractOptions) -> Result<(), String> {
    let args = build_args(input, opts, WHISPER_ARGS);
    let status = Command::new("ffmpeg")
        .args(&args)
        .arg(wav)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .stdin(Stdio::null())
        .status()
        .map_err(|e| {
            format!(
                "failed to run ffmpeg: {e} (install ffmpeg, e.g. `brew install ffmpeg` or `apt install ffmpeg`)"
            )
        })?;
    if !status.success() {
        return Err(format!("ffmpeg exited with {status}"));
    }
    Ok(())
}

const WHISPER_ARGS: &[&str] = &[
    "-map", "0:a:0", "-vn", "-ac", "1", "-ar", "16000", "-f", "wav",
];

fn build_args(input: &Path, opts: &ExtractOptions, encode: &[&str]) -> Vec<OsString> {
    let mut args: Vec<OsString> = ["-hide_banner", "-loglevel", "error", "-nostdin", "-y"]
        .iter()
        .map(OsString::from)
        .collect();
    if let Some(start) = opts.start_s {
        args.push("-ss".into());
        args.push(format!("{start}").into());
    }
    args.push("-i".into());
    args.push(input.as_os_str().to_os_string());
    if let Some(duration) = opts.duration_s {
        args.push("-t".into());
        args.push(format!("{duration}").into());
    }
    args.extend(encode.iter().map(OsString::from));
    args
}

/// Audio file formats produced by [`convert_audio`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Wav,
    Mp3,
    M4a,
}

impl AudioFormat {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "wav" | "wave" => Ok(Self::Wav),
            "mp3" | "mpeg" => Ok(Self::Mp3),
            "m4a" | "aac" | "mp4" => Ok(Self::M4a),
            other => Err(format!(
                "unknown audio format '{other}' (expected wav, mp3 or m4a)"
            )),
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
            Self::M4a => "m4a",
        }
    }

    fn encode_args(self) -> &'static [&'static str] {
        match self {
            Self::Wav => &["-map", "0:a:0", "-vn", "-c:a", "pcm_s16le"],
            Self::Mp3 => &["-map", "0:a:0", "-vn", "-c:a", "libmp3lame", "-q:a", "2"],
            Self::M4a => &["-map", "0:a:0", "-vn", "-c:a", "aac", "-b:a", "192k"],
        }
    }
}

/// Converts the audio track of `input` to `out` in the given format,
/// honouring `opts` for clip ranges. Keeps the source channel layout and
/// sample rate (unlike the 16 kHz mono WAV used for transcription).
pub fn convert_audio(
    input: &Path,
    out: &Path,
    format: AudioFormat,
    opts: &ExtractOptions,
) -> Result<(), String> {
    let args = build_args(input, opts, format.encode_args());
    let status = Command::new("ffmpeg")
        .args(&args)
        .arg(out)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .stdin(Stdio::null())
        .status()
        .map_err(|e| {
            format!(
                "failed to run ffmpeg: {e} (install ffmpeg, e.g. `brew install ffmpeg` or `apt install ffmpeg`)"
            )
        })?;
    if !status.success() {
        return Err(format!("ffmpeg exited with {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arg_strings(args: &[OsString]) -> Vec<String> {
        args.iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn default_args_have_stable_flags() {
        let args = arg_strings(&build_args(
            Path::new("in.mp4"),
            &ExtractOptions::default(),
            WHISPER_ARGS,
        ));
        for flag in ["-nostdin", "-vn", "-ac", "1", "-ar", "16000", "-f", "wav"] {
            assert!(args.iter().any(|a| a == flag), "missing {flag}");
        }
        assert!(!args.iter().any(|a| a == "-ss" || a == "-t"));
    }

    #[test]
    fn clip_range_adds_seek_and_duration() {
        let args = arg_strings(&build_args(
            Path::new("in.mp4"),
            &ExtractOptions {
                start_s: Some(12.5),
                duration_s: Some(30.0),
            },
            WHISPER_ARGS,
        ));
        let pos = |flag: &str| args.iter().position(|a| a == flag).unwrap();
        assert_eq!(args[pos("-ss") + 1], "12.5");
        assert_eq!(args[pos("-t") + 1], "30");
        // -ss must precede -i (input seeking), -t follows it.
        assert!(pos("-ss") < pos("-i") && pos("-i") < pos("-t"));
    }

    #[test]
    fn ffprobe_args_are_stable() {
        // has_audio_stream shells out to ffprobe; just ensure it doesn't
        // panic on a missing file path shape (returns Ok or Err, not panic).
        let _ = has_audio_stream(Path::new("definitely-not-here.mp4"));
    }

    #[test]
    fn audio_format_parse() {
        assert_eq!(AudioFormat::parse("mp3").unwrap(), AudioFormat::Mp3);
        assert_eq!(AudioFormat::parse("M4A").unwrap(), AudioFormat::M4a);
        assert_eq!(AudioFormat::parse("wav").unwrap(), AudioFormat::Wav);
        assert!(AudioFormat::parse("ogg").is_err());
    }

    #[test]
    fn audio_format_encode_args() {
        assert!(AudioFormat::Mp3.encode_args().contains(&"libmp3lame"));
        assert!(AudioFormat::M4a.encode_args().contains(&"aac"));
        assert!(AudioFormat::Wav.encode_args().contains(&"pcm_s16le"));
        // user-facing audio keeps source rate/channels: no -ar/-ac forcing
        for f in [AudioFormat::Wav, AudioFormat::Mp3, AudioFormat::M4a] {
            let a = f.encode_args();
            assert!(!a.contains(&"-ar") && !a.contains(&"-ac"));
        }
    }

    #[test]
    fn input_path_is_passed_verbatim() {
        let args = arg_strings(&build_args(
            Path::new("my video.mkv"),
            &ExtractOptions::default(),
            WHISPER_ARGS,
        ));
        assert!(args.iter().any(|a| a == "my video.mkv"));
    }
}
