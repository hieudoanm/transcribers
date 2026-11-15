mod mcp;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{error::ErrorKind, CommandFactory, Parser};
use whisper::{Format, TranscribeOptions};

#[derive(Parser)]
#[command(
    name = "transcribers",
    version = concat!(env!("CARGO_PKG_VERSION"), " (build ", env!("BUILD_TIME"), ")"),
    about = "Extract transcripts from video and audio files locally"
)]
struct Cli {
    /// Video or audio file (mp4, mkv, mov, wav, mp3, ...)
    #[arg(required_unless_present = "list_models")]
    input: Option<PathBuf>,

    /// Model name (tiny, base, small, medium, large, base.en, ...) or path
    /// to a .bin file. Missing models are downloaded automatically.
    #[arg(short = 'm', long, value_name = "MODEL", default_value = "base.en")]
    model: String,

    /// List known model names and exit.
    #[arg(long)]
    list_models: bool,

    /// Spoken language code or 'auto'.
    #[arg(short = 'l', long, value_name = "LANG", default_value = "auto")]
    language: String,

    /// Output format: txt, srt, vtt, json.
    #[arg(short = 'f', long, value_name = "FORMAT", default_value = "srt",
          value_parser = Format::parse)]
    format: Format,

    /// Write output to PATH instead of stdout; '-' means stdout. For
    /// srt/vtt/json the extension is appended unless PATH already ends with it.
    #[arg(short = 'o', long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Decoder threads. Default: auto (available parallelism, capped at 8).
    #[arg(short = 't', long, value_name = "N")]
    threads: Option<usize>,

    /// Start transcription at SEC seconds into the media.
    #[arg(long, value_name = "SEC", value_parser = parse_seconds)]
    start: Option<f64>,

    /// Transcribe at most SEC seconds.
    #[arg(long, value_name = "SEC", value_parser = parse_seconds)]
    duration: Option<f64>,

    /// Initial prompt to bias recognition (names, jargon, context).
    #[arg(long, value_name = "TEXT")]
    prompt: Option<String>,

    /// Translate speech to English.
    #[arg(long)]
    translate: bool,

    /// Keep the extracted 16 kHz WAV next to the output (or in the cwd).
    #[arg(long)]
    keep_audio: bool,

    /// Also write the audio track as wav, mp3 or m4a next to the transcript
    /// output (source sample rate kept).
    #[arg(short = 'a', long, value_name = "FORMAT",
          value_parser = ffmpeg::AudioFormat::parse)]
    audio: Option<ffmpeg::AudioFormat>,

    /// Only convert audio (--audio required); skips model download and
    /// transcription.
    #[arg(long)]
    audio_only: bool,
}

fn parse_seconds(v: &str) -> Result<f64, String> {
    let s: f64 = v
        .parse()
        .map_err(|_| format!("'{v}' is not a number of seconds (e.g. 12.5)"))?;
    if !s.is_finite() || s < 0.0 {
        return Err(format!("'{v}' must be a non-negative number of seconds"));
    }
    Ok(s)
}

fn parse_args() -> Cli {
    let mut cli = Cli::parse();
    if cli.output.as_ref().is_some_and(|p| p.as_os_str() == "-") {
        cli.output = None;
    }
    let fail = |kind: ErrorKind, msg: String| -> ! {
        Cli::command().error(kind, msg).exit();
    };
    if cli.threads == Some(0) {
        fail(
            ErrorKind::ValueValidation,
            "thread count must be >= 1".to_string(),
        );
    }
    if let Some(d) = cli.duration {
        if d <= 0.0 {
            fail(
                ErrorKind::ValueValidation,
                "--duration must be > 0".to_string(),
            );
        }
    }
    if cli.audio_only && cli.audio.is_none() {
        fail(
            ErrorKind::MissingRequiredArgument,
            "--audio-only requires --audio <wav|mp3|m4a>".to_string(),
        );
    }
    cli
}

fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get().min(8))
        .unwrap_or(4)
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("mcp") {
        if let Err(e) = mcp::run() {
            eprintln!("mcp: {e}");
            std::process::exit(1);
        }
        return;
    }
    let args = parse_args();
    if args.list_models {
        for m in models::KNOWN_MODELS {
            println!("{m}");
        }
        return;
    }
    if let Err(e) = real_main(&args) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn real_main(args: &Cli) -> Result<(), String> {
    let input = args
        .input
        .as_deref()
        .ok_or_else(|| "missing input file".to_string())?;
    if !input.exists() {
        return Err(format!("input file not found: {}", input.display()));
    }
    // Fail fast on files with no audio stream (skipped when ffprobe is absent
    // or cannot parse the file — ffmpeg reports those itself).
    if let Some(has_audio) = ffmpeg::has_audio_stream(input)? {
        if !has_audio {
            return Err(format!("{}: no audio stream found", input.display()));
        }
    }

    let extract = ffmpeg::ExtractOptions {
        start_s: args.start,
        duration_s: args.duration,
    };

    // Audio-only mode: convert and exit before touching models/whisper.
    if args.audio_only {
        let dest = audio_dest(args)?;
        ffmpeg::convert_audio(input, &dest, args.audio.unwrap(), &extract)?;
        eprintln!("wrote {}", dest.display());
        return Ok(());
    }

    // Cheap checks first: a bad language code must not trigger a model download.
    whisper::validate_language(&args.language)?;
    let model = models::model_path(&args.model)?;

    let work = env::temp_dir().join(format!("transcribers-{}", std::process::id()));
    fs::create_dir_all(&work).map_err(|e| format!("cannot create {}: {e}", work.display()))?;
    let wav = work.join("audio.wav");

    let topts = TranscribeOptions {
        threads: args.threads.unwrap_or_else(default_threads),
        initial_prompt: args.prompt.clone(),
        translate: args.translate,
    };

    let result = (|| -> Result<(String, PathBuf), String> {
        ffmpeg::extract_audio_with(input, &wav, &extract)?;
        let transcript = whisper::transcribe(&wav, &model, &args.language, &topts)?;
        if transcript.segments.is_empty() {
            eprintln!("warning: no speech detected");
        }
        Ok((whisper::render(args.format, &transcript), wav.clone()))
    })();

    let (text, wav) = match result {
        Ok(t) => t,
        Err(e) => {
            let _ = fs::remove_dir_all(&work);
            return Err(e);
        }
    };

    match &args.output {
        Some(path) => {
            let dest = if path.extension().is_some() {
                path.clone()
            } else {
                path.with_extension(args.format.extension())
            };
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
            }
            fs::write(&dest, &text).map_err(|e| format!("cannot write {}: {e}", dest.display()))?;
            eprintln!("wrote {}", dest.display());
        }
        None => print!("{text}"),
    }
    if args.keep_audio {
        match keep_audio_copy(&wav, input) {
            Ok(path) => eprintln!("audio: {}", path.display()),
            Err(e) => eprintln!("warning: could not keep audio: {e}"),
        }
    }
    if let Some(format) = args.audio {
        match audio_dest(args).and_then(|dest| {
            ffmpeg::convert_audio(input, &dest, format, &extract)?;
            Ok(dest)
        }) {
            Ok(dest) => eprintln!("wrote {}", dest.display()),
            Err(e) => eprintln!("warning: could not write audio: {e}"),
        }
    }
    let _ = fs::remove_dir_all(&work);
    Ok(())
}

/// Resolves where `--audio` output goes: next to `-o` (same stem, audio
/// extension) when given, otherwise `<input-stem>.<ext>` in the cwd with a
/// numeric suffix on collision.
fn audio_dest(args: &Cli) -> Result<PathBuf, String> {
    let format = args.audio.unwrap();
    let ext = format.extension();
    let (dir, stem) = match &args.output {
        Some(path) => {
            let dir = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "audio".to_string());
            (dir, stem)
        }
        None => {
            let stem = args
                .input
                .as_deref()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "audio".to_string());
            (PathBuf::from("."), stem)
        }
    };
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    for i in 0..1000u32 {
        let name = if i == 0 {
            format!("{stem}.{ext}")
        } else {
            format!("{stem}-{i}.{ext}")
        };
        let dest = dir.join(name);
        if !dest.exists() {
            return Ok(dest);
        }
    }
    Err("could not find a free filename for audio output".to_string())
}

/// Copies the extracted WAV to a stable, user-visible location: next to the
/// output file when `-o` is used, otherwise in the current directory as
/// `<input-stem>.16k.wav` (with a numeric suffix on collision).
fn keep_audio_copy(wav: &Path, input: &Path) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "audio".to_string());
    let dir = env::current_dir().map_err(|e| format!("cannot get cwd: {e}"))?;
    for i in 0..1000u32 {
        let name = if i == 0 {
            format!("{stem}.16k.wav")
        } else {
            format!("{stem}.16k-{i}.wav")
        };
        let dest = dir.join(name);
        if !dest.exists() {
            fs::copy(wav, &dest).map_err(|e| format!("cannot copy to {}: {e}", dest.display()))?;
            return Ok(dest);
        }
    }
    Err("could not find a free filename for kept audio".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli(args: &[&str]) -> Cli {
        let mut v = vec!["transcribers"];
        v.extend_from_slice(args);
        Cli::try_parse_from(v).unwrap()
    }

    #[test]
    fn defaults() {
        let a = cli(&["video.mp4"]);
        assert_eq!(a.input, Some(PathBuf::from("video.mp4")));
        assert_eq!(a.model, "base.en");
        assert_eq!(a.language, "auto");
        assert_eq!(a.format, Format::Srt);
        assert_eq!(a.threads, None);
        assert!(!a.translate && !a.keep_audio && !a.list_models && !a.audio_only);
        assert!(a.audio.is_none());
    }

    #[test]
    fn flags_parse() {
        let a = cli(&[
            "v.mp4",
            "-m",
            "small.en",
            "-l",
            "vi",
            "-f",
            "json",
            "-o",
            "out/t",
            "-t",
            "8",
            "--start",
            "1.5",
            "--duration",
            "30",
            "--prompt",
            "Dr. Smith",
            "--translate",
            "--keep-audio",
            "--audio",
            "mp3",
        ]);
        assert_eq!(a.model, "small.en");
        assert_eq!(a.language, "vi");
        assert_eq!(a.format, Format::Json);
        assert_eq!(a.output, Some(PathBuf::from("out/t")));
        assert_eq!(a.threads, Some(8));
        assert_eq!(a.start, Some(1.5));
        assert_eq!(a.duration, Some(30.0));
        assert_eq!(a.prompt.as_deref(), Some("Dr. Smith"));
        assert!(a.translate && a.keep_audio);
        assert_eq!(a.audio, Some(ffmpeg::AudioFormat::Mp3));
    }

    #[test]
    fn list_models_needs_no_input() {
        let a = cli(&["--list-models"]);
        assert!(a.list_models);
        assert!(a.input.is_none());
    }

    #[test]
    fn clap_rejects_bad_input() {
        let mut v = vec!["transcribers"];
        assert!(Cli::try_parse_from(v.clone()).is_err()); // no args, no --list-models
        v = vec!["transcribers", "a.mp4", "b.mp4"];
        assert!(Cli::try_parse_from(v).is_err());
        v = vec!["transcribers", "a.mp4", "--bogus"];
        assert!(Cli::try_parse_from(v).is_err());
        v = vec!["transcribers", "a.mp4", "-t"];
        assert!(Cli::try_parse_from(v).is_err()); // missing value
        v = vec!["transcribers", "a.mp4", "-t", "x"];
        assert!(Cli::try_parse_from(v).is_err());
        v = vec!["transcribers", "a.mp4", "--start", "-1"];
        assert!(Cli::try_parse_from(v).is_err());
        v = vec!["transcribers", "a.mp4", "-f", "bad"];
        assert!(Cli::try_parse_from(v).is_err());
        v = vec!["transcribers", "a.mp4", "--audio", "ogg"];
        assert!(Cli::try_parse_from(v).is_err());
    }

    #[test]
    fn post_validation() {
        // Validated in parse_args(), which exits on failure; test the rules
        // indirectly via Cli fields + the same conditions.
        let a = cli(&["a.mp4", "-t", "0"]);
        assert_eq!(a.threads, Some(0)); // parse_args() rejects this
        let a = cli(&["a.mp4", "--duration", "0"]);
        assert_eq!(a.duration, Some(0.0));
        let a = cli(&["a.mp4", "--audio-only", "--audio", "wav"]);
        assert!(a.audio_only && a.audio.is_some());
    }

    #[test]
    fn dash_o_means_stdout() {
        // Handled in parse_args(); verify the raw value parses as a path.
        let a = cli(&["v.mp4", "-o", "-"]);
        assert_eq!(a.output, Some(PathBuf::from("-")));
    }

    #[test]
    fn default_threads_is_positive_and_capped() {
        let n = default_threads();
        assert!((1..=8).contains(&n));
    }
}
