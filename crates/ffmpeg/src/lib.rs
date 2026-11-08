use std::path::Path;
use std::process::{Command, Stdio};

/// Extracts the audio track of `input` into a 16 kHz mono WAV at `wav`.
pub fn extract_audio(input: &Path, wav: &Path) -> Result<(), String> {
    let status = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(input)
        .args(["-vn", "-ac", "1", "-ar", "16000", "-f", "wav"])
        .arg(wav)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("failed to run ffmpeg: {e}"))?;
    if !status.success() {
        return Err(format!("ffmpeg exited with {status}"));
    }
    Ok(())
}
