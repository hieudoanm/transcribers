use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use whisper::{Format, TranscribeOptions};

struct Args {
    input: PathBuf,
    model: String,
    language: String,
    format: Format,
    output: Option<PathBuf>,
    /// `None` = auto (available parallelism, capped).
    threads: Option<usize>,
    start_s: Option<f64>,
    duration_s: Option<f64>,
    prompt: Option<String>,
    translate: bool,
    keep_audio: bool,
    list_models: bool,
    help: bool,
}

const USAGE: &str = "Usage: transcribers <input-file> [OPTIONS]\n\
         \n\
         Extracts the audio track from a video/audio file and transcribes it\n\
         with whisper.cpp (requires ffmpeg on PATH; whisper is built in).\n\
         \n\
         Arguments:\n\
         \x20 <input-file>    Video or audio file (mp4, mkv, mov, wav, mp3, ...)\n\
         \n\
         Options:\n\
         \x20 -m, --model <MODEL>       Model name (tiny, base, small, medium, large,\n\
         \x20                            base.en, ...) or path to a .bin file.\n\
         \x20                            Default: base.en. Missing models are\n\
         \x20                            downloaded automatically.\n\
         \x20     --list-models          List known model names and exit.\n\
         \x20 -l, --language <LANG>     Spoken language code or 'auto'.\n\
         \x20                            Default: auto.\n\
         \x20 -f, --format <FORMAT>     Output format: txt, srt, vtt, json.\n\
         \x20                            Default: srt (printed to stdout unless -o).\n\
         \x20 -o, --output <PATH>       Write output to PATH instead of stdout;\n\
         \x20                            '-' means stdout. For srt/vtt/json the\n\
         \x20                            extension is appended unless PATH already\n\
         \x20                            ends with it.\n\
         \x20 -t, --threads <N>         Decoder threads. Default: auto.\n\
         \x20     --start <SEC>         Start transcription at SEC seconds into\n\
         \x20                            the media (audio is still fully decoded\n\
         \x20                            from that point; timestamps are relative\n\
         \x20                            to the clip).\n\
         \x20     --duration <SEC>      Transcribe at most SEC seconds.\n\
         \x20     --prompt <TEXT>       Initial prompt to bias recognition\n\
         \x20                            (names, jargon, context).\n\
         \x20     --translate           Translate speech to English.\n\
         \x20     --keep-audio           Keep the extracted 16 kHz WAV next to\n\
         \x20                            the output (or in the cwd) and print\n\
         \x20                            its path.\n\
         \x20 -V, --version              Print version and exit.\n\
         \x20 -h, --help                 Show this help.\n";

/// Errors print usage to stderr and exit 2.
fn usage() -> ! {
    eprint!("{USAGE}");
    std::process::exit(2);
}

/// --help prints usage to stdout and exits 0, like every other CLI.
fn print_help() -> ! {
    print!("{USAGE}");
    std::process::exit(0);
}

fn parse_args() -> Result<Args, String> {
    parse_args_iter(env::args().skip(1))
}

fn parse_args_iter<I>(iter: I) -> Result<Args, String>
where
    I: Iterator<Item = String>,
{
    let mut args = Args {
        input: PathBuf::new(),
        model: "base.en".to_string(),
        language: "auto".to_string(),
        format: Format::Srt,
        output: None,
        threads: None,
        start_s: None,
        duration_s: None,
        prompt: None,
        translate: false,
        keep_audio: false,
        list_models: false,
        help: false,
    };
    let mut iter = iter.peekable();
    let mut positional = Vec::new();

    let take_value = |name: &str, iter: &mut std::iter::Peekable<I>| -> Result<String, String> {
        iter.next()
            .ok_or_else(|| format!("missing value for {name}"))
    };

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => args.help = true,
            "-V" | "--version" => {
                println!(
                    "transcribers {} (build {})",
                    env!("CARGO_PKG_VERSION"),
                    env!("BUILD_TIME")
                );
                std::process::exit(0);
            }
            "-m" | "--model" => args.model = take_value("--model", &mut iter)?,
            "--list-models" => args.list_models = true,
            "-l" | "--language" => args.language = take_value("--language", &mut iter)?,
            "-f" | "--format" => args.format = Format::parse(&take_value("--format", &mut iter)?)?,
            "-o" | "--output" => {
                let v = take_value("--output", &mut iter)?;
                args.output = if v == "-" {
                    None
                } else {
                    Some(PathBuf::from(v))
                };
            }
            "-t" | "--threads" => {
                let v = take_value("--threads", &mut iter)?;
                let n: usize = v
                    .parse()
                    .map_err(|_| format!("invalid thread count '{v}'"))?;
                if n == 0 {
                    return Err("thread count must be >= 1".to_string());
                }
                args.threads = Some(n);
            }
            "--start" => {
                let v = take_value("--start", &mut iter)?;
                args.start_s = Some(parse_seconds("--start", &v)?);
            }
            "--duration" => {
                let v = take_value("--duration", &mut iter)?;
                let d = parse_seconds("--duration", &v)?;
                if d <= 0.0 {
                    return Err("--duration must be > 0".to_string());
                }
                args.duration_s = Some(d);
            }
            "--prompt" => args.prompt = Some(take_value("--prompt", &mut iter)?),
            "--translate" => args.translate = true,
            "--keep-audio" => args.keep_audio = true,
            other if other.starts_with('-') => return Err(format!("unknown option '{other}'")),
            other => positional.push(other.to_string()),
        }
    }

    match positional.len() {
        0 if args.list_models || args.help => {}
        0 => return Err("missing input file".to_string()),
        1 => args.input = PathBuf::from(&positional[0]),
        _ => return Err(format!("unexpected argument '{}'", positional[1])),
    }
    Ok(args)
}

fn parse_seconds(flag: &str, v: &str) -> Result<f64, String> {
    let s: f64 = v
        .parse()
        .map_err(|_| format!("invalid value for {flag}: '{v}' (expected seconds, e.g. 12.5)"))?;
    if !s.is_finite() || s < 0.0 {
        return Err(format!("{flag} must be a non-negative number of seconds"));
    }
    Ok(s)
}

fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get().min(8))
        .unwrap_or(4)
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            usage();
        }
    };
    if args.help {
        print_help();
    }
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

fn real_main(args: &Args) -> Result<(), String> {
    if !args.input.exists() {
        return Err(format!("input file not found: {}", args.input.display()));
    }
    // Cheap checks first: a bad language code must not trigger a model download.
    whisper::validate_language(&args.language)?;
    let model = models::model_path(&args.model)?;

    let work = env::temp_dir().join(format!("transcribers-{}", std::process::id()));
    fs::create_dir_all(&work).map_err(|e| format!("cannot create {}: {e}", work.display()))?;
    let wav = work.join("audio.wav");

    let extract = ffmpeg::ExtractOptions {
        start_s: args.start_s,
        duration_s: args.duration_s,
    };
    let topts = TranscribeOptions {
        threads: args.threads.unwrap_or_else(default_threads),
        initial_prompt: args.prompt.clone(),
        translate: args.translate,
    };

    let result = (|| -> Result<(String, PathBuf), String> {
        ffmpeg::extract_audio_with(&args.input, &wav, &extract)?;
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
        match keep_audio_copy(&wav, &args.input) {
            Ok(path) => eprintln!("audio: {}", path.display()),
            Err(e) => eprintln!("warning: could not keep audio: {e}"),
        }
    }
    let _ = fs::remove_dir_all(&work);
    Ok(())
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

    fn parse(args: &[&str]) -> Result<Args, String> {
        parse_args_iter(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn defaults() {
        let a = parse(&["video.mp4"]).unwrap();
        assert_eq!(a.input, PathBuf::from("video.mp4"));
        assert_eq!(a.model, "base.en");
        assert_eq!(a.language, "auto");
        assert_eq!(a.format, Format::Srt);
        assert_eq!(a.threads, None);
        assert!(!a.translate && !a.keep_audio && !a.list_models);
    }

    #[test]
    fn flags_parse() {
        let a = parse(&[
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
        ])
        .unwrap();
        assert_eq!(a.model, "small.en");
        assert_eq!(a.language, "vi");
        assert_eq!(a.format, Format::Json);
        assert_eq!(a.output, Some(PathBuf::from("out/t")));
        assert_eq!(a.threads, Some(8));
        assert_eq!(a.start_s, Some(1.5));
        assert_eq!(a.duration_s, Some(30.0));
        assert_eq!(a.prompt.as_deref(), Some("Dr. Smith"));
        assert!(a.translate && a.keep_audio);
    }

    #[test]
    fn dash_o_means_stdout() {
        let a = parse(&["v.mp4", "-o", "-"]).unwrap();
        assert_eq!(a.output, None);
    }

    #[test]
    fn list_models_needs_no_input() {
        let a = parse(&["--list-models"]).unwrap();
        assert!(a.list_models);
    }

    #[test]
    fn errors() {
        assert!(parse(&[]).is_err());
        assert!(parse(&["a.mp4", "b.mp4"]).is_err());
        assert!(parse(&["a.mp4", "--bogus"]).is_err());
        assert!(parse(&["a.mp4", "-t"]).is_err());
        assert!(parse(&["a.mp4", "-t", "0"]).is_err());
        assert!(parse(&["a.mp4", "-t", "x"]).is_err());
        assert!(parse(&["a.mp4", "--start", "-1"]).is_err());
        assert!(parse(&["a.mp4", "--start", "abc"]).is_err());
        assert!(parse(&["a.mp4", "--duration", "0"]).is_err());
        assert!(parse(&["a.mp4", "-f", "bad"]).is_err());
    }

    #[test]
    fn default_threads_is_positive_and_capped() {
        let n = default_threads();
        assert!((1..=8).contains(&n));
    }
}
