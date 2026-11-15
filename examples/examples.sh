#!/usr/bin/env bash
#
# examples.sh — Transcribe every video in examples/videos into examples/transcripts.
#
# Usage:
#   ./examples/examples.sh
#   FORMAT=txt ./examples/examples.sh        # txt | srt | vtt | json
#   MODEL=small.en ./examples/examples.sh
#   AUDIO=mp3 ./examples/examples.sh          # also write audio to examples/audio
#   AUDIO_ONLY=1 AUDIO=wav ./examples/examples.sh  # audio conversion only
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT/target/release/transcribers"
VIDEOS="$ROOT/examples/videos"
OUT="$ROOT/examples/transcripts"
AUD="$ROOT/examples/audio"
FORMAT="${FORMAT:-srt}"
AUDIO="${AUDIO:-}"
AUDIO_ONLY="${AUDIO_ONLY:-0}"

if [[ ! -x "$BIN" ]]; then
    echo "error: $BIN not found; run 'cargo build --release' first" >&2
    exit 1
fi

mkdir -p "$OUT"
if [[ "$AUDIO_ONLY" == "1" && -z "$AUDIO" ]]; then
    echo "error: AUDIO_ONLY=1 requires AUDIO=<wav|mp3|m4a>" >&2
    exit 1
fi

shopt -s nullglob
files=("$VIDEOS"/*)
if [[ ${#files[@]} -eq 0 ]]; then
    echo "no files in $VIDEOS"
    exit 0
fi

failed=0
for f in "${files[@]}"; do
    [[ -f "$f" ]] || continue
    stem="$(basename "$f")"
    stem="${stem%.*}"
    echo ">> $f"
    args=(-f "$FORMAT" -o "$OUT/$stem")
    [[ -n "${MODEL:-}" ]] && args+=(--model "$MODEL")
    if [[ "$AUDIO_ONLY" == "1" ]]; then
        args=(--audio-only --audio "$AUDIO" -o "$AUD/$stem")
    fi
    "$BIN" "$f" "${args[@]}" || { echo "failed: $f" >&2; failed=$((failed + 1)); }
done

# Optional audio pass: convert each video into examples/audio/<stem>.<ext>.
if [[ -n "$AUDIO" && "$AUDIO_ONLY" != "1" ]]; then
    mkdir -p "$AUD"
    for f in "${files[@]}"; do
        [[ -f "$f" ]] || continue
        stem="$(basename "$f")"
        stem="${stem%.*}"
        echo ">> audio: $f"
        "$BIN" "$f" --audio-only --audio "$AUDIO" -o "$AUD/$stem" \
            || { echo "audio failed: $f" >&2; failed=$((failed + 1)); }
    done
fi

if [[ "$failed" -gt 0 ]]; then
    echo "$failed file(s) failed" >&2
    exit 1
fi
echo "done: transcripts in $OUT"
[[ -n "$AUDIO" ]] && echo "audio in $AUD"
