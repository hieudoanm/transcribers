# Development

## Build

```sh
cargo build --release          # binary at target/release/transcribers
cargo test                     # workspace tests
cargo clippy --workspace       # lints
```

The project is a minimal-dependency Cargo workspace: all crates return
`Result<_, String>`. whisper.cpp is compiled into the binary at build time
via `whisper-rs`/`whisper-rs-sys` (needs `cmake`); everything else shells
out to `ffmpeg`/`curl`.

## Workspace layout

```txt
transcribers/           Cargo workspace root
└── crates/
    ├── transcribers/   CLI binary — argument parsing, temp-dir orchestration
    ├── ffmpeg/         extract_audio(input, wav) — ffmpeg wrapper
    ├── models/         model_path(name) — cache resolution + HF download
    └── whisper/        whisper-rs bindings, Format enum, segment rendering
```

- `crates/ffmpeg` — owns the ffmpeg invocation (`-vn -ac 1 -ar 16000 -f wav`).
- `crates/models` — resolves a model name to `~/.cache/transcribers/models/
  ggml-<name>.bin` (or `$TRANSCRIBERS_MODEL_DIR`), downloading from
  Hugging Face when missing. Existing paths are used as-is.
- `crates/whisper` — owns the `Format` enum (`txt`/`srt`/`vtt`/`json`), WAV
  decoding to f32 samples, the `whisper-rs` transcription call, and renders
  segments to the output formats itself (no `whisper-cli` needed).
- `crates/transcribers` — hand-rolled arg parsing (no clap), temp dir
  management, stdout vs `-o` file handling.

## Install scripts

- `scripts/install.sh` — curl-able installer: tries a GitHub release binary
  (`transcribers-{os}-{arch}`), falls back to `git clone` + `cargo build
  --release` in a temp dir. Installs to `~/.local/bin` (or `$INSTALL_DIR`)
  and warns when `ffmpeg`/`curl` are missing.
- `scripts/install-local.sh` — rebuilds the local checkout only when sources
  are newer than `target/release/transcribers`, installs to `~/.local/bin`
  (or `$INSTALL_DIR`), and warns about PATH shadowing.

## Examples

`examples/videos/` contains sample media; `examples/transcripts/` receives
output. `examples/examples.sh` batch-transcribes the directory:

```sh
./examples/examples.sh
FORMAT=srt MODEL=small.en ./examples/examples.sh
```

## Conventions

- Keep crates lean; add a crate only for a clear responsibility.
- Errors are `String` messages; `main` prints `error: ...` and exits 1.
- External tools are invoked via `std::process::Command`; check
  `command -v` style availability implicitly through spawn errors.
- Model cache honours `XDG_CACHE_HOME`, then `$HOME/.cache`.
