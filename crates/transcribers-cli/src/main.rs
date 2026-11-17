mod mcp;
mod mcp_http;
mod serve;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{error::ErrorKind, ArgGroup, CommandFactory, Parser};
use whisper::{Format, TranscribeOptions};

#[derive(Parser)]
#[command(
    name = "transcribers",
    version = concat!(env!("CARGO_PKG_VERSION"), " (build ", env!("BUILD_TIME"), ")"),
    about = "Extract transcripts from video and audio files locally",
    group = ArgGroup::new("input")
        .args(["file", "url"])
        .multiple(false)
)]
struct Cli {
    /// Local video or audio file (mp4, mkv, mov, wav, mp3, ...).
    #[arg(long, value_name = "PATH")]
    file: Option<PathBuf>,

    /// http(s) URL — YouTube videos/shorts, Instagram reels, etc. —
    /// downloaded via yt-dlp first.
    #[arg(long, value_name = "URL")]
    url: Option<String>,

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

pub(crate) fn parse_seconds(v: &str) -> Result<f64, String> {
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
    if !cli.list_models && cli.file.is_none() && cli.url.is_none() {
        fail(
            ErrorKind::MissingRequiredArgument,
            "missing input: pass --file <PATH> or --url <URL>".to_string(),
        );
    }
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
    if let Some(url) = &cli.url {
        if !is_url(url) {
            fail(
                ErrorKind::ValueValidation,
                format!("--url must be an http(s) URL, got '{url}'"),
            );
        }
    }
    cli
}

pub(crate) fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get().min(8))
        .unwrap_or(4)
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("mcp") => {
            let mut http = None;
            let mut token = None;
            let mut args = std::env::args().skip(2);
            while let Some(arg) = args.next() {
                if arg == "--http" {
                    http = args.next();
                } else if let Some(value) = arg.strip_prefix("--http=") {
                    http = Some(value.to_string());
                } else if arg == "--token" {
                    token = args.next();
                } else if let Some(value) = arg.strip_prefix("--token=") {
                    token = Some(value.to_string());
                }
            }
            let result = match http {
                Some(addr) => mcp::run_http(
                    &addr,
                    token.or_else(|| std::env::var("MCP_AUTH_TOKEN").ok()),
                ),
                None => mcp::run(),
            };
            if let Err(e) = result {
                eprintln!("mcp: {e}");
                std::process::exit(1);
            }
            return;
        }
        Some("serve") => {
            if let Err(e) = serve::run() {
                eprintln!("serve: {e}");
                std::process::exit(1);
            }
            return;
        }
        _ => {}
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
    let work = env::temp_dir().join(format!("transcribers-{}", std::process::id()));
    fs::create_dir_all(&work).map_err(|e| format!("cannot create {}: {e}", work.display()))?;

    // `--url` downloads via yt-dlp into the work dir; `--file` is used
    // as-is. `media` is always a local file path from here on.
    let media = if let Some(url) = &args.url {
        download_media(url, &work)?
    } else {
        let p = args
            .file
            .clone()
            .expect("input group guarantees --file or --url");
        if !p.exists() {
            return Err(format!("input file not found: {}", p.display()));
        }
        p
    };
    let input = media.as_path();

    let extract = ffmpeg::ExtractOptions {
        start_s: args.start,
        duration_s: args.duration,
    };

    // Audio-only mode: convert and exit before touching models/whisper.
    if args.audio_only {
        // Fail fast on files with no audio stream (skipped when ffprobe is
        // absent or cannot parse the file — ffmpeg reports those itself).
        if let Some(has_audio) = ffmpeg::has_audio_stream(input)? {
            if !has_audio {
                return Err(format!("{}: no audio stream found", input.display()));
            }
        }
        let result = audio_dest(args, input).and_then(|dest| {
            ffmpeg::convert_audio(input, &dest, args.audio.unwrap(), &extract)?;
            Ok(dest)
        });
        let _ = fs::remove_dir_all(&work);
        match result {
            Ok(dest) => {
                eprintln!("wrote {}", dest.display());
                Ok(())
            }
            Err(e) => Err(e),
        }
    } else {
        let job = Job {
            model: args.model.clone(),
            language: args.language.clone(),
            format: args.format,
            threads: args.threads.unwrap_or_else(default_threads),
            start: args.start,
            duration: args.duration,
            prompt: args.prompt.clone(),
            translate: args.translate,
        };

        let result = transcribe_text(input, &work, &job);
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
                fs::write(&dest, &text)
                    .map_err(|e| format!("cannot write {}: {e}", dest.display()))?;
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
            match audio_dest(args, input).and_then(|dest| {
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
}

/// Transcription parameters shared by the CLI and the HTTP server.
pub(crate) struct Job {
    pub model: String,
    pub language: String,
    pub format: Format,
    pub threads: usize,
    pub start: Option<f64>,
    pub duration: Option<f64>,
    pub prompt: Option<String>,
    pub translate: bool,
}

/// Full transcription pipeline for a local media file: ffprobe audio-stream
/// check → language/model validation (cheap checks before any model
/// download) → 16 kHz WAV extraction into `work` → whisper → rendered text.
/// Returns `(rendered_text, wav_path)`.
pub(crate) fn transcribe_text(
    input: &Path,
    work: &Path,
    job: &Job,
) -> Result<(String, PathBuf), String> {
    // Fail fast on files with no audio stream (skipped when ffprobe is
    // absent or cannot parse the file — ffmpeg reports those itself).
    if let Some(has_audio) = ffmpeg::has_audio_stream(input)? {
        if !has_audio {
            return Err(format!("{}: no audio stream found", input.display()));
        }
    }

    // Cheap checks first: a bad language code must not trigger a model download.
    whisper::validate_language(&job.language)?;
    let model = models::model_path(&job.model)?;

    let wav = work.join("audio.wav");
    let extract = ffmpeg::ExtractOptions {
        start_s: job.start,
        duration_s: job.duration,
    };
    ffmpeg::extract_audio_with(input, &wav, &extract)?;
    let transcript = whisper::transcribe(
        &wav,
        &model,
        &job.language,
        &TranscribeOptions {
            threads: job.threads,
            initial_prompt: job.prompt.clone(),
            translate: job.translate,
        },
    )?;
    if transcript.segments.is_empty() {
        eprintln!("warning: no speech detected");
    }
    Ok((whisper::render(job.format, &transcript), wav))
}

/// Resolves where `--audio` output goes: next to `-o` (same stem, audio
/// extension) when given, otherwise `<input-stem>.<ext>` in the cwd with a
/// numeric suffix on collision.
fn audio_dest(args: &Cli, input: &Path) -> Result<PathBuf, String> {
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
            let stem = input
                .file_stem()
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

pub(crate) fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

/// Downloads the best audio-bearing media file for `url` into `work` using
/// the `yt-dlp` crate (native extractors + yt-dlp binary fallback) and
/// returns the local path. The file is named after the video title so
/// downstream naming (`--keep-audio`, `--audio`) stays sensible.
pub(crate) fn download_media(url: &str, work: &Path) -> Result<PathBuf, String> {
    eprintln!("downloading {url} ...");
    let rt =
        tokio::runtime::Runtime::new().map_err(|e| format!("cannot start async runtime: {e}"))?;
    rt.block_on(download_media_async(url, work))
}

async fn download_media_async(url: &str, work: &Path) -> Result<PathBuf, String> {
    use yt_dlp::client::deps::{Libraries, LibraryInstaller};
    use yt_dlp::Downloader;

    let ffmpeg = find_on_path("ffmpeg").ok_or_else(|| {
        "ffmpeg not found on PATH; install it (e.g. `brew install ffmpeg`)".to_string()
    })?;

    // Prefer an existing yt-dlp binary; otherwise install one into the
    // shared cache dir (only the yt-dlp binary — ffmpeg is required on
    // PATH anyway for the transcription pipeline).
    let yt_dlp = match find_on_path("yt-dlp") {
        Some(p) => p,
        None => {
            let dir = models::cache_dir().join("dlp");
            eprintln!(
                "yt-dlp not found on PATH; installing into {}",
                dir.display()
            );
            LibraryInstaller::new(dir)
                .install_youtube(None)
                .await
                .map_err(|e| format!("failed to install yt-dlp: {e}"))?
        }
    };

    let libs = Libraries::new(yt_dlp.clone(), ffmpeg);
    let downloader = Downloader::builder(libs, work.to_path_buf())
        .build()
        .await
        .map_err(|e| format!("cannot initialise downloader: {e}"))?;

    let video = match downloader.fetch_video_infos(url).await {
        Ok(video) => video,
        Err(e) => {
            // The crate's JSON model is YouTube-centric: non-YouTube sites
            // (e.g. Instagram) emit thumbnails without `preference` and
            // fail to parse. Fall back to driving the yt-dlp binary
            // directly, which handles every extractor yt-dlp supports.
            eprintln!("yt-dlp crate could not parse {url} ({e}); using yt-dlp binary");
            return download_via_binary(&yt_dlp, url, work);
        }
    };

    let stem: String = video
        .title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_");
    let stem: String = stem.chars().take(80).collect();
    let stem = if stem.is_empty() { "media" } else { &stem };
    let out = work
        .join(format!("{stem}.media"))
        .to_string_lossy()
        .into_owned();

    downloader
        .download_audio_stream_to_path(&video, &out)
        .await
        .map_err(|e| format!("failed to download audio for {url}: {e}"))
}

/// Fallback for sites the `yt-dlp` crate cannot parse: run the yt-dlp
/// binary directly with `-f bestaudio/best` and a `%(title)` output
/// template, then locate the produced file in `work`.
fn download_via_binary(yt_dlp: &Path, url: &str, work: &Path) -> Result<PathBuf, String> {
    let template = work
        .join("%(title).80s.%(ext)s")
        .to_string_lossy()
        .into_owned();
    let status = std::process::Command::new(yt_dlp)
        .args([
            "--no-playlist",
            "-f",
            "bestaudio/best",
            "--no-progress",
            "-o",
            &template,
            url,
        ])
        .status()
        .map_err(|e| format!("failed to run {}: {e}", yt_dlp.display()))?;
    if !status.success() {
        return Err(format!("yt-dlp exited with {status} for {url}"));
    }
    // The work dir is per-process and empty except for this download;
    // pick the newest regular file (skipping .part leftovers).
    let mut files: Vec<PathBuf> = fs::read_dir(work)
        .map_err(|e| format!("cannot list {}: {e}", work.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file()
                && !p
                    .file_name()
                    .map(|n| n.to_string_lossy().starts_with('.'))
                    .unwrap_or(false)
                && p.extension().map(|e| e != "part").unwrap_or(true)
        })
        .collect();
    files.sort_by_key(|p| {
        p.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    files
        .pop()
        .ok_or_else(|| format!("yt-dlp produced no file for {url}"))
}

/// Finds an executable on PATH (no shell-out).
fn find_on_path(name: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    for dir in env::split_paths(&paths) {
        let candidate = dir.join(name);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if candidate
                .metadata()
                .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
            {
                return Some(candidate);
            }
        }
        #[cfg(not(unix))]
        {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
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
        let a = cli(&["--file", "video.mp4"]);
        assert_eq!(a.file, Some(PathBuf::from("video.mp4")));
        assert_eq!(a.url, None);
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
            "--file",
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
        assert!(a.file.is_none() && a.url.is_none());
    }

    #[test]
    fn url_flag_parses() {
        let a = cli(&["--url", "https://youtube.com/watch?v=abc"]);
        assert_eq!(a.url.as_deref(), Some("https://youtube.com/watch?v=abc"));
        assert!(a.file.is_none());
    }

    #[test]
    fn clap_rejects_bad_input() {
        let mut v = vec!["transcribers"];
        assert!(Cli::try_parse_from(v.clone()).is_ok()); // no input; parse_args() rejects
        v = vec![
            "transcribers",
            "--file",
            "a.mp4",
            "--url",
            "https://x.com/v",
        ];
        assert!(Cli::try_parse_from(v).is_err()); // --file and --url conflict
        v = vec!["transcribers", "--file", "a.mp4", "b.mp4"];
        assert!(Cli::try_parse_from(v).is_err()); // stray positional
        v = vec!["transcribers", "--file", "a.mp4", "--bogus"];
        assert!(Cli::try_parse_from(v).is_err());
        v = vec!["transcribers", "--file", "a.mp4", "-t"];
        assert!(Cli::try_parse_from(v).is_err()); // missing value
        v = vec!["transcribers", "--file", "a.mp4", "-t", "x"];
        assert!(Cli::try_parse_from(v).is_err());
        v = vec!["transcribers", "--file", "a.mp4", "--start", "-1"];
        assert!(Cli::try_parse_from(v).is_err());
        v = vec!["transcribers", "--file", "a.mp4", "-f", "bad"];
        assert!(Cli::try_parse_from(v).is_err());
        v = vec!["transcribers", "--file", "a.mp4", "--audio", "ogg"];
        assert!(Cli::try_parse_from(v).is_err());
        v = vec!["transcribers", "--url", "not-a-url"];
        assert!(Cli::try_parse_from(v).is_ok()); // syntax ok; parse_args() rejects
    }

    #[test]
    fn post_validation() {
        // Validated in parse_args(), which exits on failure; test the rules
        // indirectly via Cli fields + the same conditions.
        let a = cli(&["--file", "a.mp4", "-t", "0"]);
        assert_eq!(a.threads, Some(0)); // parse_args() rejects this
        let a = cli(&["--file", "a.mp4", "--duration", "0"]);
        assert_eq!(a.duration, Some(0.0));
        let a = cli(&["--file", "a.mp4", "--audio-only", "--audio", "wav"]);
        assert!(a.audio_only && a.audio.is_some());
        let a = cli(&["--url", "ftp://x"]);
        assert_eq!(a.url.as_deref(), Some("ftp://x")); // parse_args() rejects
    }

    #[test]
    fn dash_o_means_stdout() {
        // Handled in parse_args(); verify the raw value parses as a path.
        let a = cli(&["--file", "v.mp4", "-o", "-"]);
        assert_eq!(a.output, Some(PathBuf::from("-")));
    }

    #[test]
    fn url_detection() {
        assert!(is_url("https://youtube.com/shorts/abc"));
        assert!(is_url("http://example.com/v.mp4"));
        assert!(!is_url("video.mp4"));
        assert!(!is_url("/tmp/video.mp4"));
    }

    #[test]
    fn default_threads_is_positive_and_capped() {
        let n = default_threads();
        assert!((1..=8).contains(&n));
    }
}
