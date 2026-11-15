#!/usr/bin/env bash
#
# examples.sh — Transcribe example inputs into examples/transcripts (and
# examples/audio when AUDIO is set).
#
# Inputs: every file in examples/videos/ plus every URL in examples/urls.txt
# (one per line, '#' comments). Pass arguments to use an explicit input list
# instead (files or URLs).
#
# Usage:
#   ./examples/examples.sh
#   ./examples/examples.sh "https://youtube.com/watch?v=..." file.mp4
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

# Derive a filesystem-safe stem for an input (file path or URL).
input_stem() {
    local input="$1"
    case "$input" in
        http://*|https://*)
            # YouTube: watch?v=ID, youtu.be/ID, /shorts/ID, /live/ID
            if [[ "$input" =~ (v=|youtu\.be/|shorts/|live/)([A-Za-z0-9_-]{6,}) ]]; then
                echo "yt-${BASH_REMATCH[2]}"; return
            fi
            # Instagram: /reel(s)/CODE, /p/CODE, /tv/CODE
            if [[ "$input" =~ /(reels?|p|tv)/([A-Za-z0-9_-]+) ]]; then
                echo "ig-${BASH_REMATCH[2]}"; return
            fi
            basename "${input%%\?*}" | tr -cd 'A-Za-z0-9_-'
            ;;
        *)
            stem="$(basename "$input")"
            echo "${stem%.*}"
            ;;
    esac
}

if [[ $# -gt 0 ]]; then
    inputs=("$@")
else
    shopt -s nullglob
    inputs=("$VIDEOS"/*)
    if [[ -f "$ROOT/examples/urls.txt" ]]; then
        while IFS= read -r line; do
            line="${line%%#*}"
            line="$(echo "$line" | xargs)"
            [[ -n "$line" ]] && inputs+=("$line")
        done < "$ROOT/examples/urls.txt"
    fi
fi

if [[ ${#inputs[@]} -eq 0 ]]; then
    echo "no inputs in $VIDEOS or examples/urls.txt"
    exit 0
fi

failed=0
for f in "${inputs[@]}"; do
    [[ -f "$f" || "$f" == http://* || "$f" == https://* ]] || continue
    stem="$(input_stem "$f")"
    echo ">> $f"
    args=(-f "$FORMAT" -o "$OUT/$stem")
    [[ -n "${MODEL:-}" ]] && args+=(--model "$MODEL")
    if [[ "$AUDIO_ONLY" == "1" ]]; then
        args=(--audio-only --audio "$AUDIO" -o "$AUD/$stem")
    fi
    if [[ "$f" == http://* || "$f" == https://* ]]; then
        "$BIN" --url "$f" "${args[@]}" || { echo "failed: $f" >&2; failed=$((failed + 1)); }
    else
        "$BIN" --file "$f" "${args[@]}" || { echo "failed: $f" >&2; failed=$((failed + 1)); }
    fi
done

# Optional audio pass: convert each input into examples/audio/<stem>.<ext>.
if [[ -n "$AUDIO" && "$AUDIO_ONLY" != "1" ]]; then
    mkdir -p "$AUD"
    for f in "${inputs[@]}"; do
        [[ -f "$f" || "$f" == http://* || "$f" == https://* ]] || continue
        stem="$(input_stem "$f")"
        echo ">> audio: $f"
        if [[ "$f" == http://* || "$f" == https://* ]]; then
            "$BIN" --url "$f" --audio-only --audio "$AUDIO" -o "$AUD/$stem" \
                || { echo "audio failed: $f" >&2; failed=$((failed + 1)); }
        else
            "$BIN" --file "$f" --audio-only --audio "$AUDIO" -o "$AUD/$stem" \
                || { echo "audio failed: $f" >&2; failed=$((failed + 1)); }
        fi
    done
fi

if [[ "$failed" -gt 0 ]]; then
    echo "$failed file(s) failed" >&2
    exit 1
fi
echo "done: transcripts in $OUT"
if [[ -n "$AUDIO" ]]; then
    echo "audio in $AUD"
fi
