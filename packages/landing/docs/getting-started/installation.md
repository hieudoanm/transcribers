# Installation

## Prebuilt binary

```bash
curl -fsSL https://raw.githubusercontent.com/hieudoanm/transcribers/main/scripts/install.sh | bash
```

Falls back to a source build when no prebuilt artifact matches. Installs to
`~/.local/bin` by default; override with `INSTALL_DIR`.

Prebuilt releases exist for macOS (arm64) and Linux (x86_64).

## From a local checkout

```bash
./scripts/install-local.sh
```

## From source

```bash
cargo build --release        # also prints the binary size
./target/release/transcribers --version
cargo test --workspace
```

## Requirements

- `ffmpeg` on `PATH` (`brew install ffmpeg`)
- `curl` for model downloads
- Building from source needs `cmake` and a C++ toolchain

whisper.cpp is compiled into the binary at build time — no separate whisper
install. Models (`tiny` through `large-v3`, multilingual and `.en` variants)
download automatically on first use into `~/.cache/transcribers/models/`.
