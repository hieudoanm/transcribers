# transcribers-whisper

In-process whisper.cpp transcription and output formatting for
`transcribers`, via [`whisper-rs`](https://crates.io/crates/whisper-rs).
whisper.cpp is compiled from vendored source (cmake) and statically linked
at build time — no `whisper-cli` binary is needed at runtime. Build
requires `cmake` and a C++ toolchain.

Lib name is `whisper` (package `transcribers-whisper`).

## API

```rust
pub enum Format { Txt, Srt, Vtt, Json }
impl Format {
    pub fn parse(s: &str) -> Result<Self, String>;
    pub fn extension(self) -> &'static str;
}

pub struct Segment { pub start_ms: i64, pub end_ms: i64, pub text: String }
pub struct Transcript {
    pub language: Option<String>,
    pub duration_ms: i64,
    pub segments: Vec<Segment>,
}
pub struct TranscribeOptions {
    pub threads: usize,               // 0 = whisper.cpp default
    pub initial_prompt: Option<String>,
    pub translate: bool,
}

pub fn validate_language(language: &str) -> Result<(), String>;
pub fn transcribe(audio: &Path, model: &Path, language: &str,
                  opts: &TranscribeOptions) -> Result<Transcript, String>;
pub fn render(format: Format, transcript: &Transcript) -> String;
```

## `transcribe`

- Input must be **16 kHz mono PCM16 WAV** (as produced by
  `ffmpeg::extract_audio_with`); `read_wav_f32` walks RIFF chunks and
  rejects wrong rate, non-mono, non-PCM16, or missing `fmt `/`data` chunks.
- Decodes to `f32`, runs `WhisperState::full` with greedy sampling
  (`best_of: 5`), `threads`, `initial_prompt`, `translate`, and
  `language` (`"auto"` → `set_language(None)`).
- Progress is reported to stderr as `transcribing... N%` via
  `set_progress_callback_safe`; whisper.cpp's own logs are silenced with a
  no-op `set_log_callback` (`ggml_log_level` typed via `whisper-rs-sys` —
  required because bindings differ per platform).
- Returns `Transcript` with detected `language` (`full_lang_id_from_state`
  → `get_lang_str`) and `duration_ms` (samples / 16).

## `render`

| `Format` | flag/ext | output |
|----------|----------|--------|
| `Txt` | `txt` | one segment per line, no timestamps |
| `Srt` | `srt` | SubRip, `HH:MM:SS,mmm` |
| `Vtt` | `vtt` | WebVTT header + `HH:MM:SS.mmm` |
| `Json` | `json` | `{"language","duration_ms","segments":[{start,end,text}]}` |

Timestamp helpers (`srt_ts`/`vtt_ts`/`split_ts`) and `json_escape` are
private — covered by `tests/render.rs`.

## Tests

`cargo test -p transcribers-whisper` — `tests/render.rs` covers `Format`
parsing, all four renderers, JSON escaping, and null language. No whisper
model needed (rendering only).
