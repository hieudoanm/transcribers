#!/usr/bin/env bash
set -e

REPO="hieudoanm/transcribers"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

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
if [ "$ARCH" = "x86_64" ]; then
  ARCH="x86_64"
elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
  ARCH="aarch64"
fi

BINARY_URL="https://github.com/${REPO}/releases/latest/download/transcribers-${OS}-${ARCH}"
if command -v curl >/dev/null 2>&1; then
  if curl -fsSL "$BINARY_URL" -o /tmp/transcribers 2>/dev/null; then
    chmod +x /tmp/transcribers
    sudo mv /tmp/transcribers "${INSTALL_DIR}/transcribers" 2>/dev/null || mv /tmp/transcribers "${INSTALL_DIR}/transcribers"
    echo "transcribers installed to ${INSTALL_DIR}/transcribers"
    "${INSTALL_DIR}/transcribers" --version || true
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
trap 'rm -rf "$TMP_DIR"' EXIT
cd "$TMP_DIR"
git clone --depth 1 "https://github.com/${REPO}.git" transcribers
cd transcribers
cargo build --release
sudo cp target/release/transcribers "${INSTALL_DIR}/transcribers" 2>/dev/null || cp target/release/transcribers "${INSTALL_DIR}/transcribers"
echo "transcribers installed to ${INSTALL_DIR}/transcribers"
"${INSTALL_DIR}/transcribers" --version || true
check_deps
