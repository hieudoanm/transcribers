## transcribers — latest build

Extract transcripts from video/audio files locally: ffmpeg audio
extraction → whisper.cpp transcription → `txt`/`srt`/`vtt`/`json`
output. Models auto-download on first use. Includes an MCP server.

### Download

| Platform | Asset |
| --- | --- |
| Linux x86_64 | `transcribers-x86_64-unknown-linux-gnu` |
| macOS Apple Silicon | `transcribers-aarch64-apple-darwin` |
| Windows x86_64 | `transcribers-x86_64-pc-windows-msvc.exe` |

Runtime dependency: `ffmpeg` on PATH.

### Quick start

```sh
transcribers talk.mp4 -o talk.txt
transcribers talk.mp4 --format srt --language en
transcribers mcp
```

### Documentation

- [README](https://github.com/hieudoanm/transcribers#readme)
