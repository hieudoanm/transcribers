# transcribers documentation

**transcribers** extracts transcripts from video and audio files locally —
a single Rust binary powered by `ffmpeg` for audio extraction and
`whisper.cpp` for transcription. Everything runs offline; media never leaves
your machine.

## Sections

- **Getting started** — install the binary and transcribe your first file.
- **Guides** — command usage, local development, and the MCP server.

## Output formats

Plain text, SRT, WebVTT subtitles, or JSON with timestamps.

An MCP server (`transcribers mcp`) exposes transcription and model listing to
MCP clients — see [MCP server](guides/mcp-server.md).
