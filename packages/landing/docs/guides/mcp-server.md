# MCP server

`transcribers` ships a [Model Context Protocol](https://modelcontextprotocol.io)
server over stdio. MCP clients can transcribe media and list models through
the same local pipeline as the CLI — audio never leaves your machine.

## Run

```sh
transcribers mcp
```

The server reads newline-delimited JSON-RPC on stdin and writes responses on
stdout. It delegates every tool call to the `transcribers` binary itself.

## Client configuration

```json
{
  "mcpServers": {
    "transcribers": {
      "command": "transcribers",
      "args": ["mcp"]
    }
  }
}
```

`ffmpeg` must be on `PATH` and models download on first use, exactly as with
the CLI.

## Tools

| Tool            | Description                                                          |
| --------------- | -------------------------------------------------------------------- |
| `transcribe`    | Transcribe a video/audio file or URL to txt, srt, vtt, or json              |
| `list_models`   | List known whisper model names                                       |

`transcribe` accepts `input` (required — a file path or an http(s) URL) plus optional `model`, `language`,
`format`, `output`, `threads`, `start`, `duration`, `prompt`, and `translate`
— the same options as the command line. See [Usage](usage.md) for details.
