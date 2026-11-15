# Development

## Build

```sh
cargo build --release          # binary at target/release/transcribers
cargo test --workspace         # unit + integration tests
cargo clippy --workspace --all-targets
cargo fmt --all -- --check
make build                     # release build + prints binary size
```

Minimal-dependency Cargo workspace (`resolver = "2"`,
`workspace.package.version = 0.0.1`). Library crates return
`Result<_, String>`; the CLI prints `error: ...` and maps failures to exit
codes (1 runtime, 2 usage). whisper.cpp is compiled into the binary at
build time via `whisper-rs`/`whisper-rs-sys` — first build needs `cmake`
and a C++ toolchain and takes noticeably longer than a plain Rust build.

## Workspace layout

```txt
transcribers/
├── Cargo.toml                # [workspace] only; shared version/edition
├── crates/
│   ├── transcribers-cli/     # package transcribers-cli → bin `transcribers`
│   │   ├── src/main.rs       #   clap Cli, validation, real_main pipeline
│   │   ├── src/mcp.rs        #   MCP stdio JSON-RPC server (delegates to CLI)
│   │   └── build.rs          #   stamps BUILD_TIME for --version
│   ├── transcribers-ffmpeg/  # lib `ffmpeg`: extract/probe/convert audio
│   ├── transcribers-models/  # lib `models`: KNOWN_MODELS, model_path
│   └── transcribers-whisper/ # lib `whisper`: whisper-rs, Format, render
├── scripts/                  # install.sh (release/curl), install-local.sh
├── examples/                 # videos/, transcripts/, audio/, examples.sh
├── docs/                     # usage.md, development.md (this file)
├── packages/landing/         # static GitHub Pages site
└── assets/branding/          # svg + png logo/og; svg-to-png.sh regenerates
```

Package names carry the `transcribers-` prefix; lib target names stay
short (`ffmpeg`, `models`, `whisper`) so call sites read naturally. The
CLI depends on them via `package = "transcribers-*"`.

### Crate responsibilities

- **transcribers-cli** — clap derive arg parsing (`Cli`), post-parse
  validation (exit 2 via `Cli::command().error`), yt-dlp URL downloads
  (tokio runtime, auto-installs the yt-dlp binary into
  `~/.cache/transcribers/dlp` when absent), temp-dir lifecycle,
  output destination rules, `mcp` subcommand, `--version` build stamp.
- **transcribers-ffmpeg** — all ffmpeg/ffprobe invocations:
  `extract_audio_with` (16 kHz mono WAV, `WHISPER_ARGS`),
  `has_audio_stream` (ffprobe; `Ok(None)` = skip check),
  `convert_audio`/`AudioFormat` (wav/mp3/m4a at source rate).
- **transcribers-models** — `KNOWN_MODELS` allowlist,
  `normalize_model_name`, cache/download logic (`TRANSCRIBERS_MODEL_DIR` →
  XDG → `~/.cache`; `curl -fSL -C -` + `ggml` magic verification).
- **transcribers-whisper** — whisper.cpp bindings via `whisper-rs`,
  `read_wav_f32` (enforces 16 kHz mono PCM16), `TranscribeOptions`,
  `Transcript`, `render` for txt/srt/vtt/json, log silencing via
  `whisper_rs_sys::ggml_log_level`.

## Tests

- `crates/transcribers-cli` — `#[cfg(test)]` module: clap parsing via
  `Cli::try_parse_from`, flag coverage, thread defaults.
- `crates/transcribers-ffmpeg` — `build_args` flag ordering and
  `AudioFormat` encode args.
- `crates/transcribers-models` — `normalize_model_name`, `KNOWN_MODELS`.
- `crates/transcribers-whisper` — `tests/render.rs` integration tests for
  all output formats (no model needed).

## Install scripts

- `scripts/install.sh` — curl-able installer: downloads the matching
  `transcribers-{os}-{arch}` asset from the `latest` GitHub release,
  falls back to `git clone` + `cargo build --release` in a temp dir.
  Installs to `~/.local/bin` (or `$INSTALL_DIR`), verifies with
  `--version`, warns when `ffmpeg`/`curl`/`cmake` are missing.
- `scripts/install-local.sh` — rebuilds the local checkout when sources
  are newer than `target/release/transcribers`, installs to
  `~/.local/bin` (or `$INSTALL_DIR`), warns about PATH shadowing.

## Examples

`examples/videos/` holds sample media (`*.mp4` gitignored);
`examples/transcripts/` receives transcript output; `examples/audio/`
receives optional audio conversions. `examples/examples.sh` batch-runs the
CLI over the directory:

```sh
./examples/examples.sh                          # srt transcripts
FORMAT=json MODEL=small.en ./examples/examples.sh
AUDIO=mp3 ./examples/examples.sh                # + audio pass -> examples/audio
AUDIO_ONLY=1 AUDIO=wav ./examples/examples.sh   # audio only
```

## CI

`.github/workflows/ci.yaml`: `cli-build` matrix (macos/ubuntu/windows)
runs fmt+clippy+test on the ubuntu leg and release-builds all three;
`release` recreates the `latest` GitHub release with README notes on
pushes to `master`; `landing-*` jobs deploy `packages/landing` to Pages.

## Conventions

- Keep crates lean; add a crate only for a clear responsibility.
- Errors are `String` messages; diagnostics go to stderr.
- External tools via `std::process::Command`; check availability through
  spawn errors (ffprobe absence must degrade gracefully).
- Model cache honours `TRANSCRIBERS_MODEL_DIR`, then `XDG_CACHE_HOME`,
  then `$HOME/.cache`.
- Update `README.md` and `docs/` when behaviour changes.
