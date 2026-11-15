# transcribers-ffmpeg

Audio extraction and conversion for `transcribers`, wrapping the `ffmpeg`
(and `ffprobe`) CLIs via `std::process::Command`.

Lib name is `ffmpeg` (package `transcribers-ffmpeg`), so call sites keep
using `ffmpeg::`.

## API

```rust
pub struct ExtractOptions {
    pub start_s: Option<f64>,    // -ss before -i (input seek)
    pub duration_s: Option<f64>, // -t after -i
}

pub fn extract_audio(input: &Path, wav: &Path) -> Result<(), String>
pub fn extract_audio_with(input: &Path, wav: &Path, opts: &ExtractOptions)
    -> Result<(), String>
pub fn has_audio_stream(input: &Path) -> Result<Option<bool>, String>

pub enum AudioFormat { Wav, Mp3, M4a }
impl AudioFormat { pub fn parse(s: &str) -> Result<Self, String>;
                   pub fn extension(self) -> &'static str; }
pub fn convert_audio(input: &Path, out: &Path, format: AudioFormat,
                     opts: &ExtractOptions) -> Result<(), String>
```

## Whisper path

`extract_audio*` always produces **16 kHz mono PCM WAV** — the exact format
whisper.cpp expects:

```sh
ffmpeg -hide_banner -loglevel error -nostdin -y \
    [-ss <start>] -i <input> [-t <duration>] \
    -map 0:a:0 -vn -ac 1 -ar 16000 -f wav <wav>
```

These flags live in the private `WHISPER_ARGS` constant — do not weaken
them; the WAV reader in `transcribers-whisper` validates 16 kHz/mono/PCM16
and rejects anything else.

## User-facing audio (`convert_audio`)

Keeps the **source** sample rate and channel layout (no `-ac`/`-ar`):

| Format | Encode args |
|--------|-------------|
| `wav`  | `-c:a pcm_s16le` |
| `mp3`  | `-c:a libmp3lame -q:a 2` |
| `m4a`  | `-c:a aac -b:a 192k` |

Clip ranges (`--start`/`--duration`) apply to both paths.

## Probing

`has_audio_stream` runs `ffprobe -select_streams a -show_entries
stream=codec_type -of csv=p=0`. Returns `Ok(Some(bool))` on a definitive
answer, `Ok(None)` when ffprobe is missing or cannot parse the input —
callers must treat `None` as "skip the check" and let ffmpeg report the
real error.

## Requirements / notes

- `ffmpeg` on `PATH`; `ffprobe` optional (ships with ffmpeg).
- ffmpeg's stderr is inherited so its diagnostics reach the user;
  stdout/stdin are silenced (`-nostdin`, `Stdio::null`).
- Zero dependencies; errors are `String`s.
- Tests cover `build_args` flag ordering (`-ss` before `-i`, `-t` after)
  and `AudioFormat` parsing/encode args.
