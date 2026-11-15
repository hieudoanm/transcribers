# transcribers-cli

The `transcribers` binary — a local CLI that extracts transcripts from video
and audio files. Pipeline: `ffmpeg` extracts 16 kHz mono WAV → whisper.cpp
(statically linked via `transcribers-whisper`) transcribes → output rendered
as `txt`/`srt`/`vtt`/`json`, optionally alongside a converted audio file
(`wav`/`mp3`/`m4a`).

## CLI

```
transcribers (--file <PATH> | --url <URL>) [OPTIONS]
transcribers mcp            # MCP server over stdio JSON-RPC
transcribers serve [--host H] [--port P]   # HTTP API (default 127.0.0.1:8080)
```

`--file` takes a local media path; `--url` takes an `http(s)` URL
(YouTube videos/shorts, Instagram reels, ...) downloaded via the `yt-dlp`
crate first. The two are mutually exclusive and one is required (except
with `--list-models`).

Options (see `--help` for the authoritative list):

| Flag | Default | Purpose |
|------|---------|---------|
| `--file` | — | Local video/audio file path |
| `--url` | — | `http(s)` URL to download then transcribe |
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

Argument parsing uses `clap` (derive); `--file`/`--url` live in a
non-required `ArgGroup` so `--list-models` needs no input. Post-parse
validation lives in `parse_args()`: an input is required unless
`--list-models`, `--url` must be `http(s)`, threads ≥ 1, `--duration > 0`,
`--audio-only` requires `--audio`; violations exit 2 with a clap-style
error.

## Pipeline (`real_main`)

0. `--url` inputs are downloaded via the `yt-dlp`
   crate (tokio runtime; `Libraries`/`Downloader` API). A `yt-dlp`
   binary on PATH is used when present, else one is installed into
   `~/.cache/transcribers/dlp`. If the crate cannot parse the site's
   metadata (its JSON model is YouTube-centric — e.g. Instagram
   thumbnails lack `preference`), it falls back to running the yt-dlp
   binary directly (`-f bestaudio/best`, `%(title)` output template).
   Files land in the work dir named after the video title.
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

## Server (`serve.rs`)

`transcribers serve` runs an axum HTTP server. `POST /transcribe` takes
`multipart/form-data`: a `file` upload **or** a `url` field, plus optional
`model`/`language`/`format`/`threads`/`start`/`duration`/`prompt`/
`translate` text fields (all strings; parsed with the same rules as the
CLI). The handler collects fields, validates cheaply (400 on bad input),
then runs `download_media`/`transcribe_text` in `spawn_blocking` inside a
per-request `transcribers-serve-<pid>-<n>` temp dir. Response is the
rendered transcript (`application/json` for `format=json`). `GET /` serves
Swagger UI (CDN-hosted `swagger-ui-dist`) backed by the checked-in spec in
`assets/openapi.{yaml,json}` — also exposed at `/openapi.json` and
`/openapi.yaml`. `GET /health` returns `ok`; `DefaultBodyLimit` is disabled
since uploads stream to disk.

`Job` + `transcribe_text` are the shared core used by both `real_main` and
the server — keep them in sync with the CLI flags.

## MCP

`transcribers mcp` runs a minimal MCP server over stdio JSON-RPC
(`src/mcp.rs`). Tools shell out to the same `transcribers` binary so MCP
behaviour always matches the CLI. Protocol version `2024-11-05`.

## Tests

`cargo test -p transcribers-cli` — clap parsing (`Cli::try_parse_from`),
flag coverage, and `default_threads` bounds.
