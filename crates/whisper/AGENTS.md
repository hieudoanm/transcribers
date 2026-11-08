# AGENTS.md — whisper

In-process whisper.cpp transcription via `whisper-rs` (statically linked at
build time; needs `cmake`).

- Public API: `Format::{parse, extension}`, `Segment`,
  `transcribe(audio, model, language, threads) -> Result<Transcript, String>`,
  `render(format, segments) -> String`.
- `transcribe` takes a 16 kHz mono PCM WAV path, decodes to `f32` itself,
  and returns segments with millisecond timestamps.
- `render` owns all output formatting (txt/srt/vtt/json); keep timestamp
  helpers (`srt_ts`/`vtt_ts`/`split_ts`) and `json_escape` correct.
- whisper.cpp logs are silenced via `set_log_callback` — keep the no-op
  callback; do not let whisper.cpp spam stderr.
- Language `"auto"` must map to `set_language(None)`.
- Errors are `String`s.
