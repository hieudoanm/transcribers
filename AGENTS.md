# AGENTS.md

Guide for coding agents working in this repository.

## Project

`transcribers` is a Rust CLI that extracts transcripts from video/audio files.
Pipeline: `ffmpeg` extracts 16 kHz mono WAV → whisper.cpp (compiled in via
`whisper-rs`) transcribes → output as `txt`/`srt`/`vtt`/`json` on stdout or
to a file.

## Commands

```sh
cargo build --release     # binary: target/release/transcribers
cargo test
cargo clippy --workspace
./examples/examples.sh    # batch-transcribe examples/videos -> examples/transcripts
```

Runtime dep on `PATH`: `ffmpeg` only (brew: `ffmpeg`).
whisper.cpp is statically linked via `whisper-rs`/`whisper-rs-sys` at build
time (needs `cmake`). Models auto-download to
`~/.cache/transcribers/models/`.

## Structure

Cargo workspace; only external dep is `whisper-rs` (compiles whisper.cpp
into the binary at build time):

- `crates/transcribers` — CLI binary: args + orchestration
- `crates/ffmpeg` — audio extraction
- `crates/models` — model resolution/download
- `crates/whisper` — whisper.cpp bindings (`whisper-rs`), `Format` enum,
  segment rendering (txt/srt/vtt/json), WAV decoding
- `scripts/install.sh`, `scripts/install-local.sh` — installers
- `examples/` — sample videos and batch script
- `docs/` — usage and development docs

## Conventions

- Keep crates lean; errors are `Result<_, String>`.
- whisper crate expects 16 kHz mono PCM WAV; `read_wav_f32` rejects other
  rates/channel counts. Unit tests live in `crates/*/tests/` and inline
  `#[cfg(test)]` modules.
- Each crate has one responsibility; don't merge crates without reason.
- Shell out via `std::process::Command`, never reimplement ffmpeg/whisper logic.
- Update `README.md` and `docs/` when behaviour changes.
