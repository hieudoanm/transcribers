# transcribers

<p align="left">
  <img src="assets/png/logo.png" alt="transcribers logo" width="96">
</p>

CLI to extract transcripts from video/audio files (mp4, mkv, mov, wav, mp3, ...).

It extracts the audio track with `ffmpeg`, then transcribes it with
[whisper.cpp](https://github.com/ggml-org/whisper.cpp), which is compiled
into the binary at build time via [`whisper-rs`](https://crates.io/crates/whisper-rs)
— no separate whisper installation needed.

## Install

```sh
# from a release binary (falls back to building from source)
curl -fsSL https://raw.githubusercontent.com/hieudoanm/transcribers/main/scripts/install.sh | bash

# or from a local checkout
./scripts/install-local.sh
```

Both install to `/usr/local/bin` (`install.sh`) or `~/.local/bin`
(`install-local.sh`); override with `INSTALL_DIR`.

## Requirements

- `ffmpeg` on `PATH` (`brew install ffmpeg`)
- Building from source needs `cmake` and a C++ toolchain (whisper.cpp is
  compiled in automatically)
- Model files (`.bin`) are downloaded automatically on first use into
  `~/.cache/transcribers/models/`

## Usage

```sh
cargo build --release

# plain transcript to stdout
./target/release/transcribers examples/videos/ai-unslop-design.mp4

# subtitles file
./target/release/transcribers examples/videos/ai-unslop-design.mp4 -f srt -o out/ai-unslop-design

# other options
./target/release/transcribers video.mp4 --model small.en --language en --format vtt
```

Options: `-m/--model` (default `base.en`), `-l/--language` (default `auto`),
`-f/--format` (`txt`, `srt`, `vtt`, `json`), `-o/--output`, `--keep-audio`.

Model names (`tiny`, `base`, `small`, `medium`, `large`, `*.en`, ...) are
downloaded from <https://huggingface.co/ggerganov/whisper.cpp>; you can also
pass a path to a local `.bin` model file.

## Examples

`examples/videos/` holds sample inputs and `examples/transcripts/` the
generated output. Re-run the batch script with:

```sh
./examples/examples.sh            # FORMAT=srt MODEL=small.en also supported
```

## Documentation

- [docs/usage.md](docs/usage.md) — CLI options, formats and model selection
- [docs/development.md](docs/development.md) — workspace layout, build and install scripts
- [AGENTS.md](AGENTS.md) — project guide for coding agents
