# AGENTS.md — ffmpeg

Single-purpose crate: extract an audio track to 16 kHz mono WAV via the
`ffmpeg` CLI.

- Public API: `extract_audio(input: &Path, wav: &Path) -> Result<(), String>`
- Keep the ffmpeg flags stable (`-vn -ac 1 -ar 16000 -f wav`); whisper.cpp
  depends on this exact format.
- No dependencies; errors are `String`s.
- Do not add probing (`ffprobe`) here unless a caller needs it.
