# Contributing to transcribers

Thanks for contributing to transcribers, a Rust CLI that extracts transcripts
from video/audio files: ffmpeg extracts 16 kHz mono WAV, whisper.cpp
(statically linked via `whisper-rs`) transcribes, output is txt/srt/vtt/json.

## Repository layout

- `crates/transcribers` — CLI binary: args + orchestration.
- `crates/ffmpeg` — audio extraction.
- `crates/models` — model resolution/download.
- `crates/whisper` — whisper.cpp bindings and segment rendering.
- `scripts/` — install helpers; `examples/` — sample videos and batch script.

## Setup

```bash
git clone https://github.com/hieudoanm/transcribers.git
cd transcribers
cargo build --release
```

Runtime dependency: `ffmpeg` on `PATH`. Building requires `cmake` (whisper.cpp
is compiled in). Models auto-download to `~/.cache/transcribers/models/`.

## Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Conventions

- Keep crates lean; errors are `Result<_, String>`.
- whisper crate expects 16 kHz mono PCM WAV; `read_wav_f32` rejects other
  rates/channel counts.
- Shell out to `ffmpeg` via `std::process::Command` — never reimplement
  ffmpeg/whisper logic.
- Update `README.md` and `docs/` when behaviour changes.

## License

By contributing, you agree that your contributions will be licensed under the
repository license in [`LICENSE`](LICENSE).
