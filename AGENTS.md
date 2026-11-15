# AGENTS.md

Guide for coding agents working in this repository.

## Project

`transcribers` is a Rust CLI that extracts transcripts from video/audio
files, locally. Pipeline:

1. `ffprobe` checks the input has an audio stream (skipped if ffprobe is
   absent or cannot parse the file).
2. `ffmpeg` extracts 16 kHz mono PCM WAV to a per-PID temp dir.
3. whisper.cpp — compiled into the binary via `whisper-rs` — transcribes
   in-process.
4. Output rendered as `txt`/`srt`/`vtt`/`json` to stdout or `-o`.
5. Optional: `--audio` writes wav/mp3/m4a via a second ffmpeg pass;
   `--audio-only` does just that and exits before model resolution.

## Commands

```sh
cargo build --release     # binary: target/release/transcribers (prints size)
cargo test --workspace    # unit + integration tests
cargo clippy --workspace --all-targets
cargo fmt --all -- --check
./examples/examples.sh    # batch: FORMAT=, MODEL=, AUDIO=, AUDIO_ONLY= env vars
make build|test|clippy|fmt|run|install|clean
```

Runtime dep on `PATH`: `ffmpeg` only (`ffprobe` optional but recommended;
`brew install ffmpeg`). URL inputs use the `yt-dlp` crate; a `yt-dlp`
binary on PATH is preferred, otherwise one is auto-installed into
`~/.cache/transcribers/dlp`. `curl` needed once per model download. Building
from source needs `cmake` + a C++ toolchain (whisper.cpp compiles via
cmake inside `whisper-rs-sys`).

## Structure

Cargo workspace (`resolver = "2"`, `workspace.package.version = 0.0.1`):

- `crates/transcribers-cli` — package `transcribers-cli`, binary
  `transcribers`. clap derive parsing (`Cli`), post-parse validation,
  yt-dlp URL downloads (tokio runtime),
  temp-dir orchestration in `real_main`, `build.rs` stamps `BUILD_TIME`,
  `src/mcp.rs` is an MCP stdio JSON-RPC server that shells out to the CLI.
- `crates/transcribers-ffmpeg` — lib name `ffmpeg`. `extract_audio*`
  (16 kHz mono WAV via `WHISPER_ARGS`), `has_audio_stream` (ffprobe),
  `convert_audio` + `AudioFormat` (wav/mp3/m4a, source rate kept).
- `crates/transcribers-models` — lib name `models`. `KNOWN_MODELS`,
  `normalize_model_name`, `model_path` (validate → cache lookup → curl
  download with `-C -` resume + `ggml` magic check). Model dir:
  `$TRANSCRIBERS_MODEL_DIR` → `$XDG_CACHE_HOME/transcribers/models` →
  `~/.cache/transcribers/models`.
- `crates/transcribers-whisper` — lib name `whisper`. `whisper-rs` +
  `whisper-rs-sys` deps; `Format`, `Segment`, `Transcript`,
  `TranscribeOptions`, `validate_language`, `transcribe`, `render`,
  `read_wav_f32` (16 kHz mono PCM16 only).
- `scripts/install.sh`, `scripts/install-local.sh` — installers
  (`INSTALL_DIR`, default `~/.local/bin`; verify via `--version`).
- `examples/` — sample videos (`*.mp4` ignored), `examples.sh` batch script,
  `transcripts/` output, `audio/` optional audio output.
- `docs/` — `usage.md` (CLI reference), `development.md` (build/layout).
- `packages/landing/` — static GitHub Pages site (own AGENTS.md).
- `assets/branding/{svg,png}` — logo/og art; `assets/svg-to-png.sh` converts.

## License

GPL-3.0-only — required by the `yt-dlp` crate dependency. Do not
relicense or add copyleft-incompatible deps without discussing.

## Conventions

- Keep crates lean; one responsibility each; errors are `Result<_, String>`.
- Lib crates keep short lib names (`ffmpeg`, `models`, `whisper`) while
  package names carry the `transcribers-` prefix — preserve both.
- whisper path requires exactly 16 kHz mono PCM16 WAV; `read_wav_f32`
  enforces it — do not relax `WHISPER_ARGS` or the reader.
- `convert_audio` must not force `-ac`/`-ar`; user-facing audio keeps the
  source layout.
- Diagnostics/progress go to stderr; stdout carries only transcript data.
- clap owns usage errors (exit 2) and help/version (exit 0); runtime
  errors exit 1. `--version` format `transcribers <ver> (build <ts>)` is
  parsed by install scripts — do not change it.
- `silence_log` in the whisper crate must keep the
  `whisper_rs_sys::ggml_log_level` parameter type (platform-dependent).
- Tests: `crates/*/tests/` for integration, `#[cfg(test)]` modules for
  unit tests (cli arg parsing, ffmpeg arg building, model name
  normalisation, render formats).
- Shell out via `std::process::Command`; never reimplement ffmpeg/whisper.
- Update `README.md`, `docs/`, and the relevant crate README/AGENTS when
  behaviour changes; keep the landing page demo output in sync with real
  CLI output.

## CI (`.github/workflows/ci.yaml`)

Jobs: `cli-build` (matrix macos/ubuntu/windows; fmt+clippy+test gated to
the ubuntu leg, then release build + artifact), `release` (pushes to
`master` delete and recreate the `latest` GitHub release from README
notes), `landing-build`/`landing-deploy` (GitHub Pages from
`packages/landing`, base path `/transcribers/`).
