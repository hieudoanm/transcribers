# How to install the transcribers CLI on Linux

For x86_64 Linux distributions. The audio transcription CLI — it needs `ffmpeg`
on your `PATH` for decoding

## Option 1: install script (recommended)

```sh
curl -fsSL https://github.com/hieudoanm/transcribers/master/scripts/install.sh | bash
```

The script detects `linux/x86_64` and downloads
`cli-transcribers-x86_64-unknown-linux-gnu` from the
[latest release](https://github.com/hieudoanm/transcribers/releases/latest). It
installs to `~/.local/bin/transcribers` — override with `INSTALL_DIR=/path`.

## Option 2: manual download

1. Open the
   [downloads page](https://hieudoanm.github.io/transcribers/downloads/) and
   download **Linux** under **CLI** —
   `cli-transcribers-x86_64-unknown-linux-gnu`.
2. Make it executable and move it onto your `PATH`:

```sh
chmod +x cli-transcribers-x86_64-unknown-linux-gnu
mv cli-transcribers-x86_64-unknown-linux-gnu ~/.local/bin/transcribers
```

3. Verify:

```sh
transcribers --version
```

## Troubleshooting

- `transcribers: command not found` — ensure `~/.local/bin` is on your `PATH`
  (most distributions already include it).
- `No such file or directory` on a glibc system — the binary is built against
  glibc; on musl-based distros (Alpine) build from source instead.
