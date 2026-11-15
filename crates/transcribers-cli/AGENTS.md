# AGENTS.md — transcribers-cli

The `transcribers` binary crate (package `transcribers-cli`, bin `transcribers`).

- CLI parsing is `clap` derive (`Cli` in `src/main.rs`); keep flag names,
  defaults and doc comments in sync with `docs/usage.md`, `README.md`, and
  the landing page demo.
- `real_main` order matters: input exists → ffprobe audio-stream check →
  audio-only early exit → language validation → model resolution → extract →
  transcribe → render → optional audio/keep-audio → temp cleanup. Do not
  move model download ahead of the cheap checks.
- Errors are `Result<_, String>`; user-facing diagnostics go to stderr so
  stdout stays clean for `-o -` transcript output.
- `src/mcp.rs` must keep delegating to the `transcribers` binary — never
  reimplement transcription logic there.
- `build.rs` provides `BUILD_TIME`; `--version` output format is
  `transcribers <ver> (build <ts>)` — do not break it (install scripts parse it).
- Post-parse validation exits via `Cli::command().error(...)`; keep exit
  code 2 for usage errors and 1 for runtime errors.
