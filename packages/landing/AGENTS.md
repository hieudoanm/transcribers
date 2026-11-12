# AGENTS.md - packages/landing

## Overview

Static marketing/landing page for transcribers. A single `index.html` +
`styles.css` — no framework, no build step, no JavaScript.
Deployed to GitHub Pages as-is.

## Conventions

- **Fonts:** Inter (sans) loaded via the Google Fonts `<link>` in `index.html`; mono stack `ui-monospace, 'SF Mono', SFMono-Regular, Menlo, Consolas, monospace`. Tokens: `--font-sans` / `--font-mono`. Do not add other typefaces.

- Keep it dependency-free: edit `index.html`/`styles.css` directly.
- Asset URLs are relative (`favicon.svg`, `og-image.png`, `styles.css`) so
  the page works under the `/transcribers/` Pages base path and from
  `file://`.
- Light theme only; design follows the "AI unslop" rules (see
  `hieudoanm/notes/ai-unslop-design.md`): no filler badges, monospace only
  for code/commands, one accent colour, real CLI output instead of
  illustrations.
- Terminal/output demos must reflect real `transcribers` output —
  regenerate them if the CLI output format changes.
- Keep it a landing page — no upload UI or server logic.

## Commands

There are none — serve statically:

```bash
python3 -m http.server 8000 -d packages/landing
```
