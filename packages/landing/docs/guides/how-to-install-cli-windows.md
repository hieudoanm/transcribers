# How to install the transcribers CLI on Windows

For 64-bit Windows. The audio transcription CLI — it needs `ffmpeg` on your
`PATH` for decoding

The release binary is a native `.exe`. The install script is bash, so on Windows
run it from **Git Bash** or **WSL** — or download the binary manually.

## Option 1: install script (recommended)

```sh
curl -fsSL https://github.com/hieudoanm/transcribers/master/scripts/install.sh | bash
```

The script detects Windows and downloads
`cli-transcribers-x86_64-pc-windows-msvc.exe`, installing it to
`~/.local/bin/transcribers` (or the path you pass as `INSTALL_DIR`).

## Option 2: manual download

1. Open the
   [downloads page](https://hieudoanm.github.io/transcribers/downloads/) and
   download **Windows** under **CLI** —
   `cli-transcribers-x86_64-pc-windows-msvc.exe`.
2. Rename it to `transcribers.exe` and place it in a directory on your `PATH`
   (for example `%LOCALAPPDATA%\Programs\transcribers\` or `C:\tools\`).
3. Open a new terminal and verify:

```powershell
transcribers --version
```

## Troubleshooting

- `transcribers is not recognized` — the directory is not on `PATH`. Add it via
  Settings → Environment Variables, or call the binary by full path.
- SmartScreen warning — the binary is unsigned. Choose **More info → Run
  anyway**.
