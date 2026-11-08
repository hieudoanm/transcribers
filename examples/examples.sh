#!/usr/bin/env bash
#
# examples.sh — Transcribe every video in examples/videos into examples/transcripts.
#
# Usage:
#   ./examples/examples.sh
#   FORMAT=txt ./examples/examples.sh        # txt | srt | vtt | json
#   MODEL=small.en ./examples/examples.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT/target/release/transcribers"
VIDEOS="$ROOT/examples/videos"
OUT="$ROOT/examples/transcripts"
FORMAT="${FORMAT:-srt}"

if [[ ! -x "$BIN" ]]; then
    echo "error: $BIN not found; run 'cargo build --release' first" >&2
    exit 1
fi

mkdir -p "$OUT"

shopt -s nullglob
files=("$VIDEOS"/*)
if [[ ${#files[@]} -eq 0 ]]; then
    echo "no files in $VIDEOS"
    exit 0
fi

for f in "${files[@]}"; do
    [[ -f "$f" ]] || continue
    stem="$(basename "$f")"
    stem="${stem%.*}"
    echo ">> $f"
    args=(-f "$FORMAT" -o "$OUT/$stem")
    [[ -n "${MODEL:-}" ]] && args+=(--model "$MODEL")
    "$BIN" "$f" "${args[@]}"
done

echo "done: transcripts in $OUT"
