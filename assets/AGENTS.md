# AGENTS.md — assets

Rules for working in `assets/`.

## Sources of truth

- `branding/svg/*.svg` are the canonical logo/OG sources. `branding/png/`
  is generated output — never edit PNGs directly.
- `demo/svg/demo.svg` is the animated terminal demo (SMIL `<animate>`);
  `demo/svg/terminal.svg` is its static final frame. `demo/png/` and
  `demo/gif/` are generated.

## After editing an SVG

1. Run `assets/svg-to-png.sh` (requires `rsvg-convert`) to regenerate the
   sibling `png/` outputs.
2. If the SVG feeds the landing page (`branding/*`), run
   `assets/sync-assets.sh` to update `packages/landing/public/`.
3. `demo.gif` has no in-repo generator — flag to the user that it needs
   regeneration when `demo.svg` changes.

## Content rules

- Terminal/demo text must match real `transcribers` behaviour:
  - Commands use flags: `transcribers --file video.mp4`,
    `transcribers --url "https://…"`, `transcribers serve`, `transcribers mcp`.
    There is **no positional input argument**.
  - `--version` output is `transcribers <ver> (build <YYYY.MM.DD.hh.mm.ss>)`.
  - Diagnostics (`wrote ...`, `transcribing... N%`, `downloading ...`) go to
    stderr; stdout carries only the transcript.
- Keep the demo dark-terminal aesthetic: `#0b0b0b` background, `#141414`
  window, `#ff8a5c` prompt, `#7dd3a8` success, mono font stack.
- `og.svg` is 1280x640 (2:1); `logo.svg`/favicon mark is 360x360 centered on
  a 512x512 canvas with the 32x32 grid and 18px effective stroke.
