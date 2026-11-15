# AGENTS.md — transcribers-whisper

In-process whisper.cpp transcription via `whisper-rs` (statically linked at
build time; needs `cmake` + C++ toolchain).

- Lib name is `whisper`; package is `transcribers-whisper`.
- Public API: `Format::{parse, extension}`, `Segment`, `Transcript`,
  `TranscribeOptions`, `validate_language`,
  `transcribe(audio, model, language, opts) -> Result<Transcript, String>`,
  `render(format, transcript) -> String`.
- `transcribe` takes a 16 kHz mono PCM16 WAV path and decodes to `f32`
  itself; `read_wav_f32` must keep rejecting non-16 kHz / non-mono /
  non-PCM16 input — that is the contract with `transcribers-ffmpeg`'s
  `WHISPER_ARGS`.
- `render` owns all output formatting (txt/srt/vtt/json); keep timestamp
  helpers (`srt_ts`/`vtt_ts`/`split_ts`) and `json_escape` correct — tests
  live in `tests/render.rs`, do not move them inline.
- whisper.cpp logs are silenced via `set_log_callback` — keep the no-op
  callback; `silence_log` must use `whisper_rs_sys::ggml_log_level` as the
  level type (bindings differ between macOS pregenerated and bindgen
  output on other platforms — a concrete int type breaks one of them).
- Language `"auto"` must map to `set_language(None)`; `validate_language`
  runs before any model download in the CLI.
- Errors are `String`s.
