#!/usr/bin/env bash
set -euo pipefail

REPO="hieudoanm/transcribers"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

check_deps() {
  missing=0
  for tool in ffmpeg curl; do
    if ! command -v "$tool" >/dev/null 2>&1; then
      echo "warning: '$tool' not found on PATH" >&2
      missing=1
    fi
  done
  if [ "$missing" -eq 1 ]; then
    echo "transcribers requires ffmpeg (and curl for model downloads)." >&2
    echo "On macOS: brew install ffmpeg" >&2
  fi
}

echo "Installing transcribers..."

# Try to download prebuilt binary if available
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64 | amd64) ARCH="x86_64" ;;
  arm64 | aarch64) ARCH="aarch64" ;;
esac

case "$OS" in
  linux)  TRIPLE="${ARCH}-unknown-linux-gnu" ;;
  darwin) TRIPLE="${ARCH}-apple-darwin" ;;
  *)      TRIPLE="x86_64-pc-windows-msvc" ;;
esac

BINARY_URL="https://github.com/${REPO}/releases/latest/download/cli-transcribers-${TRIPLE}"
[[ "$OS" != "linux" && "$OS" != "darwin" ]] && BINARY_URL="${BINARY_URL}.exe"
TMP_BIN="$(mktemp -t transcribers.XXXXXX)"
trap 'rm -f "$TMP_BIN"' EXIT

if command -v curl >/dev/null 2>&1; then
  if curl -fsSL --connect-timeout 15 --retry 3 -o "$TMP_BIN" "$BINARY_URL" 2>/dev/null \
      && [ -s "$TMP_BIN" ]; then
    chmod +x "$TMP_BIN"
    mkdir -p "$INSTALL_DIR"
    install -m 0755 "$TMP_BIN" "$INSTALL_DIR/transcribers" 2>/dev/null \
      || mv "$TMP_BIN" "$INSTALL_DIR/transcribers"
    echo "transcribers installed to $INSTALL_DIR/transcribers"
    "$INSTALL_DIR/transcribers" --version || true
    check_deps
    exit 0
  fi
fi

# Fallback: build from source with cargo
if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust toolchain not found. Please install Rust from https://rustup.rs"
  exit 1
fi
if ! command -v cmake >/dev/null 2>&1; then
  echo "cmake not found; whisper.cpp is compiled at build time (brew install cmake)"
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR" "$TMP_BIN"' EXIT
cd "$TMP_DIR"
git clone --depth 1 "https://github.com/${REPO}.git" transcribers
cd transcribers
cargo build --release
mkdir -p "$INSTALL_DIR"
install -m 0755 target/release/transcribers "$INSTALL_DIR/transcribers"
echo "transcribers installed to $INSTALL_DIR/transcribers"
"$INSTALL_DIR/transcribers" --version || true
check_deps
