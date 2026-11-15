# AGENTS.md — transcribers-ffmpeg

Single-purpose crate: ffmpeg/ffprobe wrappers for audio extraction,
probing, and conversion.

- Lib name is `ffmpeg`; package is `transcribers-ffmpeg`.
- Public API: `ExtractOptions`, `extract_audio`, `extract_audio_with`,
  `has_audio_stream`, `AudioFormat`, `convert_audio`.
- `WHISPER_ARGS` (`-map 0:a:0 -vn -ac 1 -ar 16000 -f wav`) is a hard
  contract with `transcribers-whisper`'s WAV reader — never change it.
- `convert_audio` must NOT force `-ac`/`-ar`; user-facing audio keeps the
  source layout. Per-format encode args live in `AudioFormat::encode_args`.
- `has_audio_stream` returns `Ok(None)` (not an error) when ffprobe is
  absent or fails — callers skip the check and let ffmpeg report errors.
- Keep `-ss` before `-i` (input seeking) and `-t` after it; tests assert
  this ordering.
- Zero dependencies; errors are `String`s; shell out via `Command` only.
