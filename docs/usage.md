# Usage

```sh
transcribers <input-file> [OPTIONS]
```

Extracts the audio track from a video/audio file and transcribes it with
whisper.cpp (compiled into the binary). Requires `ffmpeg` on `PATH`.

## Arguments

| Argument       | Description                                        |
| -------------- | -------------------------------------------------- |
| `<input-file>` | Video or audio file (mp4, mkv, mov, wav, mp3, ...) |

## Options

| Option                    | Default   | Description                                                 |
| ------------------------- | --------- | ----------------------------------------------------------- |
| `-m`, `--model <MODEL>`   | `base.en` | Model name or path to a `.bin` file                         |
| `-l`, `--language <LANG>` | `auto`    | Spoken language code or `auto` for detection                |
| `-f`, `--format <FORMAT>` | `srt`     | Output format: `txt`, `srt`, `vtt`, `json`                  |
| `-o`, `--output <PATH>`   | stdout    | Write output to `PATH` instead of stdout                    |
| `--keep-audio`            | off       | Keep the extracted WAV file (path printed to stderr/stdout) |
| `-h`, `--help`            | —         | Show help                                                   |

## Examples

```sh
# srt transcript to stdout (default)
transcribers video.mp4

# write a subtitles file next to nothing in particular
transcribers video.mp4 -f srt -o out/video
# -> writes out/video.srt

# keep the extracted 16 kHz mono WAV for inspection
transcribers video.mp4 --keep-audio

# force English with a bigger model
transcribers video.mp4 --model small.en --language en
```

When `-o` is given without a file extension, the format's extension is
appended (`-f srt -o out/video` → `out/video.srt`). When `-o` already has an
extension it is used as-is.

## Output formats

- `txt` — plain text, one segment per line, no timestamps
- `srt` — SubRip subtitles with timestamps
- `vtt` — WebVTT subtitles
- `json` — whisper.cpp JSON output (segments with offsets and tokens when
  produced by `-oj`)

## Models

Model names map to `ggml-<name>.bin` files downloaded automatically on first
use into `~/.cache/transcribers/models/` (`$XDG_CACHE_HOME/transcribers/models`
when set). Already-downloaded models are reused — no further network access.

Common choices:

| Name       | Size   | Notes                               |
| ---------- | ------ | ----------------------------------- |
| `tiny`     | ~75MB  | fastest, lowest accuracy            |
| `base.en`  | ~141MB | default; English-only               |
| `small.en` | ~465MB | better accuracy, still English-only |
| `base`     | ~141MB | multilingual                        |
| `small`    | ~465MB | multilingual                        |
| `medium`   | ~1.5GB | high accuracy                       |
| `large-v3` | ~3.1GB | best accuracy, slow                 |

Pass a full path to any local `.bin` file to use a custom or quantized model.

## Batch transcription

`examples/examples.sh` transcribes everything in `examples/videos/` into
`examples/transcripts/`:

```sh
./examples/examples.sh
FORMAT=srt ./examples/examples.sh
MODEL=small.en ./examples/examples.sh
```

## Pipeline

1. `ffmpeg` extracts the audio track to 16 kHz mono WAV in a temp dir.
2. whisper.cpp (linked via `whisper-rs`) transcribes the WAV in-process
   with the given model and language.
3. Segments are rendered to the requested format (`txt`/`srt`/`vtt`/`json`)
   and streamed to stdout or written to `-o`.

The temp dir is removed afterwards unless `--keep-audio` is passed.
