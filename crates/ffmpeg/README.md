# ffmpeg

Audio extraction crate for `transcribers`. Wraps the `ffmpeg` CLI to pull the
audio track out of a video or audio file as 16 kHz mono WAV — the format
whisper.cpp expects.

## API

```rust
pub fn extract_audio(input: &Path, wav: &Path) -> Result<(), String>
```

Runs:

```sh
ffmpeg -hide_banner -loglevel error -y -i <input> \
    -vn -ac 1 -ar 16000 -f wav <wav>
```

- `-vn` drops all video streams
- `-ac 1 -ar 16000` produces mono 16 kHz audio

## Requirements

`ffmpeg` must be on `PATH`. Errors are returned as `Err(String)`; ffmpeg's
own stderr is inherited so failures are visible to the user.

## Notes

- Zero dependencies; uses `std::process::Command` only.
- The caller owns the output path and temp-dir lifecycle.
