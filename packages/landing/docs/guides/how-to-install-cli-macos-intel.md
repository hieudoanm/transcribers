# How to install the transcribers CLI on macOS (Intel)

For Intel-based Macs. The audio transcription CLI — it needs `ffmpeg` on your
`PATH` for decoding

## Option 1: install script (recommended)

```sh
curl -fsSL https://github.com/hieudoanm/transcribers/master/scripts/install.sh | bash
```

The script detects `macos/x86_64` and downloads
`cli-transcribers-x86_64-apple-darwin` from the
[latest release](https://github.com/hieudoanm/transcribers/releases/latest). It
installs to `~/.local/bin/transcribers` — override with `INSTALL_DIR=/path`.

## Option 2: manual download

1. Open the
   [downloads page](https://hieudoanm.github.io/transcribers/downloads/) and
   download **macOS (Intel)** under **CLI** —
   `cli-transcribers-x86_64-apple-darwin`.
2. Make it executable and move it onto your `PATH`:

```sh
chmod +x cli-transcribers-x86_64-apple-darwin
mv cli-transcribers-x86_64-apple-darwin ~/.local/bin/transcribers
```

3. Verify:

```sh
transcribers --version
```

## Troubleshooting

- `transcribers: command not found` — the install directory is not on your
  `PATH`. Add `export PATH="$HOME/.local/bin:$PATH"` to your shell profile.
- Gatekeeper warning — the binary is unsigned. Right-click → Open, or run
  `xattr -d com.apple.quarantine` on the installed binary.
