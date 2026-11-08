use std::env;
use std::fs;
use std::path::PathBuf;

use whisper::Format;

struct Args {
    input: PathBuf,
    model: String,
    language: String,
    format: Format,
    output: Option<PathBuf>,
    keep_audio: bool,
    help: bool,
}

fn usage() -> ! {
    eprintln!(
        "Usage: transcribe <input-file> [OPTIONS]\n\
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
         \x20 -l, --language <LANG>     Spoken language code or 'auto'.\n\
         \x20                            Default: auto.\n\
         \x20 -f, --format <FORMAT>     Output format: txt, srt, vtt, json.\n\
         \x20                            Default: srt (printed to stdout unless -o).\n\
         \x20 -o, --output <PATH>       Write output to PATH instead of stdout.\n\
         \x20                            For srt/vtt/json the extension is appended\n\
         \x20                            unless PATH already ends with it.\n\
         \x20     --keep-audio           Keep the extracted WAV file (printed path).\n\
         \x20 -V, --version              Print version and exit.\n\
         \x20 -h, --help                 Show this help."
    );
    std::process::exit(2);
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        input: PathBuf::new(),
        model: "base.en".to_string(),
        language: "auto".to_string(),
        format: Format::Srt,
        output: None,
        keep_audio: false,
        help: false,
    };
    let mut iter = env::args().skip(1);
    let mut positional = Vec::new();

    while let Some(arg) = iter.next() {
        let mut take_value = |name: &str| -> Result<String, String> {
            iter.next()
                .ok_or_else(|| format!("missing value for {name}"))
        };
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
            "-m" | "--model" => args.model = take_value("--model")?,
            "-l" | "--language" => args.language = take_value("--language")?,
            "-f" | "--format" => args.format = Format::parse(&take_value("--format")?)?,
            "-o" | "--output" => args.output = Some(PathBuf::from(take_value("--output")?)),
            "--keep-audio" => args.keep_audio = true,
            other if other.starts_with('-') => return Err(format!("unknown option '{other}'")),
            other => positional.push(other.to_string()),
        }
    }

    match positional.len() {
        0 => return Err("missing input file".to_string()),
        1 => args.input = PathBuf::from(&positional[0]),
        _ => return Err(format!("unexpected argument '{}'", positional[1])),
    }
    Ok(args)
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
        usage();
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
    let model = models::model_path(&args.model)?;

    let work = env::temp_dir().join(format!("transcribers-{}", std::process::id()));
    fs::create_dir_all(&work).map_err(|e| format!("cannot create {}: {e}", work.display()))?;
    let wav = work.join("audio.wav");

    ffmpeg::extract_audio(&args.input, &wav)?;
    let segments = whisper::transcribe(&wav, &model, &args.language)?;
    let text = whisper::render(args.format, &segments);

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
            println!("wrote {}", dest.display());
        }
        None => print!("{text}"),
    }
    if args.format == Format::Txt && args.output.is_none() && !text.ends_with('\n') {
        println!();
    }

    if args.keep_audio {
        println!("audio: {}", wav.display());
    } else {
        let _ = fs::remove_dir_all(&work);
    }
    Ok(())
}
