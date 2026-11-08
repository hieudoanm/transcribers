#!/usr/bin/env bash
#
# svg-to-png.sh — Convert SVG assets to PNG.
#
# Usage:
#   ./assets/svg-to-png.sh                 # convert all of assets/svg/*.svg
#   ./assets/svg-to-png.sh logo            # convert one asset by name
#   SCALE=2 ./assets/svg-to-png.sh         # 2x resolution
#
# Output goes to assets/png/<name>.png. Requires one of: rsvg-convert,
# magick/convert, or qlmanage (macOS).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SVG_DIR="$ROOT/assets/svg"
PNG_DIR="$ROOT/assets/png"
SCALE="${SCALE:-1}"

mkdir -p "$PNG_DIR"

if command -v rsvg-convert >/dev/null 2>&1; then
    convert() {
        rsvg-convert -w "$((WIDTH * SCALE))" -h "$((HEIGHT * SCALE))" "$1" -o "$2"
    }
elif command -v magick >/dev/null 2>&1; then
    convert() {
        magick -background none -density "$((96 * SCALE))" "$1" -resize "${WIDTH}x${HEIGHT}" "$2"
    }
elif command -v convert >/dev/null 2>&1; then
    convert() {
        convert -background none -density "$((96 * SCALE))" "$1" -resize "${WIDTH}x${HEIGHT}" "$2"
    }
elif command -v qlmanage >/dev/null 2>&1; then
    convert() {
        qlmanage -t -s "$((WIDTH * SCALE))" -o "$PNG_DIR" "$1" >/dev/null 2>&1
        mv "$PNG_DIR/$(basename "$1" .svg).png" "$2"
    }
else
    echo "error: need rsvg-convert, ImageMagick, or qlmanage" >&2
    exit 1
fi

# width/height per asset (used by renderers that need explicit output size)
width_of() {
    case "$(basename "$1")" in
        logo.svg) echo 512 ;;
        og.svg)   echo 1280 ;;
        *)        echo 512 ;;
    esac
}
height_of() {
    case "$(basename "$1")" in
        logo.svg) echo 512 ;;
        og.svg)   echo 640 ;;
        *)        echo 512 ;;
    esac
}

if [[ $# -gt 0 ]]; then
    svgs=()
    for name in "$@"; do
        svgs+=("$SVG_DIR/${name%.svg}.svg")
    done
else
    shopt -s nullglob
    svgs=("$SVG_DIR"/*.svg)
fi

for svg in "${svgs[@]}"; do
    [[ -f "$svg" ]] || { echo "skip (missing): $svg" >&2; continue; }
    name="$(basename "$svg" .svg)"
    WIDTH="$(width_of "$svg")"
    HEIGHT="$(height_of "$svg")"
    out="$PNG_DIR/$name.png"
    convert "$svg" "$out"
    echo "wrote $out"
done
