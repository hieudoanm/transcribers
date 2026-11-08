# whisper

In-process whisper.cpp transcription and output formatting for
`transcribers`, via [`whisper-rs`](https://crates.io/crates/whisper-rs).
whisper.cpp is compiled from source and statically linked at build time —
no `whisper-cli` binary is required at runtime.

## API

```rust
pub enum Format { Txt, Srt, Vtt, Json }

impl Format {
    pub fn parse(s: &str) -> Result<Self, String>;
    pub fn extension(self) -> &'static str;
}

pub struct Segment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

pub fn transcribe(audio: &Path, model: &Path, language: &str)
    -> Result<Vec<Segment>, String>
pub fn render(format: Format, segments: &[Segment]) -> String
```

`transcribe` expects a 16 kHz mono PCM WAV (as produced by
`ffmpeg::extract_audio`), decodes it to `f32` samples, runs
`WhisperState::full` with greedy decoding, and returns timed segments.

`render` produces the final output:

| `Format` | flag/extension | output                                      |
|----------|----------------|---------------------------------------------|
| `Txt`    | `txt`          | one segment per line, no timestamps         |
| `Srt`    | `srt`          | SubRip with `HH:MM:SS,mmm` timestamps       |
| `Vtt`    | `vtt`          | WebVTT with `HH:MM:SS.mmm` timestamps       |
| `Json`   | `json`         | `{"segments":[{start,end,text}]}` (ms)      |

`Format::parse` accepts `txt`/`text`, `srt`, `vtt`, `json` (case-insensitive).

## Build requirements

`cmake` and a C++ toolchain — `whisper-rs-sys` compiles whisper.cpp during
`cargo build`. whisper.cpp's log output is silenced via a no-op log callback.

## Notes

- Language `"auto"` maps to whisper's language auto-detection.
- Zero runtime dependencies beyond `ffmpeg` on `PATH`.
