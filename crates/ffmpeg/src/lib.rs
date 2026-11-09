use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};

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
    let args = build_args(input, opts);
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

fn build_args(input: &Path, opts: &ExtractOptions) -> Vec<OsString> {
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
    args.extend(
        [
            "-map", "0:a:0", "-vn", "-ac", "1", "-ar", "16000", "-f", "wav",
        ]
        .iter()
        .map(OsString::from),
    );
    args
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
        let args = arg_strings(&build_args(Path::new("in.mp4"), &ExtractOptions::default()));
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
        ));
        let pos = |flag: &str| args.iter().position(|a| a == flag).unwrap();
        assert_eq!(args[pos("-ss") + 1], "12.5");
        assert_eq!(args[pos("-t") + 1], "30");
        // -ss must precede -i (input seeking), -t follows it.
        assert!(pos("-ss") < pos("-i") && pos("-i") < pos("-t"));
    }

    #[test]
    fn input_path_is_passed_verbatim() {
        let args = arg_strings(&build_args(
            Path::new("my video.mkv"),
            &ExtractOptions::default(),
        ));
        assert!(args.iter().any(|a| a == "my video.mkv"));
    }
}
