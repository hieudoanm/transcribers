# transcribers-cli

The `transcribers` binary — a local CLI that extracts transcripts from video
and audio files. Pipeline: `ffmpeg` extracts 16 kHz mono WAV → whisper.cpp
(statically linked via `transcribers-whisper`) transcribes → output rendered
as `txt`/`srt`/`vtt`/`json`, optionally alongside a converted audio file
(`wav`/`mp3`/`m4a`).

## CLI

```
transcribers <input-file> [OPTIONS]
transcribers mcp            # MCP server over stdio JSON-RPC
```

Options (see `--help` for the authoritative list):

| Flag | Default | Purpose |
|------|---------|---------|
| `-m`, `--model` | `base.en` | Model name or path to a `.bin` file |
| `--list-models` | — | List known model names and exit |
| `-l`, `--language` | `auto` | Language code or `auto` detection |
| `-f`, `--format` | `srt` | `txt`, `srt`, `vtt`, `json` |
| `-o`, `--output` | stdout | Output path (`-` = stdout) |
| `-t`, `--threads` | auto | Decoder threads (parallelism, cap 8) |
| `--start` | `0` | Start offset in seconds |
| `--duration` | end | Max seconds to transcribe |
| `--prompt` | — | Initial prompt to bias recognition |
| `--translate` | off | Translate speech to English |
| `--keep-audio` | off | Keep the extracted 16 kHz WAV |
| `-a`, `--audio` | — | Also write audio as `wav`/`mp3`/`m4a` |
| `--audio-only` | off | Convert audio only; skips transcription |
| `-V`, `--version` | — | Version + build timestamp |
| `-h`, `--help` | — | Help |

Argument parsing uses `clap` (derive). Post-parse validation lives in
`parse_args()`: threads ≥ 1, `--duration > 0`, `--audio-only` requires
`--audio`; violations exit 2 with a clap-style error.

## Pipeline (`real_main`)

1. `ffmpeg::has_audio_stream` — `ffprobe` check; fails fast with
   `no audio stream found` (skipped when ffprobe is absent).
2. Audio-only mode short-circuits here: `ffmpeg::convert_audio` → exit.
3. `whisper::validate_language`, then `models::model_path` (downloads the
   model on first use — only reached for real transcriptions).
4. `ffmpeg::extract_audio_with` → 16 kHz mono WAV in a per-PID temp dir.
5. `whisper::transcribe` → `Transcript`; warns on empty segments.
6. `whisper::render` → stdout or `-o` file (extension appended when missing).
7. Optional `--keep-audio` copy and `--audio` conversion; temp dir removed
   unless `--keep-audio`. Stale `transcribers-*` temp dirs (> 1 h) are
   cleaned at startup.

`build.rs` stamps `BUILD_TIME` (UTC, `YYYY.MM.DD.hh.mm.ss`) into the binary;
`--version` prints `transcribers <version> (build <BUILD_TIME>)`.

## MCP

`transcribers mcp` runs a minimal MCP server over stdio JSON-RPC
(`src/mcp.rs`). Tools shell out to the same `transcribers` binary so MCP
behaviour always matches the CLI. Protocol version `2024-11-05`.

## Tests

`cargo test -p transcribers-cli` — clap parsing (`Cli::try_parse_from`),
flag coverage, and `default_threads` bounds.
