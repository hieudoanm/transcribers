# assets

Branding, demo media, and generated images for the `transcribers` repo.
Everything here is source material — the PNGs/GIFs are generated, never
hand-edited.

## Layout

```
assets/
├── branding/
│   ├── svg/           # source SVGs: logo.svg (512x512), og.svg (1280x640)
│   └── png/           # rendered PNGs (logo.png 512x512, og.png 1280x640)
├── demo/
│   ├── svg/           # demo.svg (animated terminal, 1280x640 SMIL)
│   │                  # terminal.svg (static final frame)
│   ├── png/           # terminal.png rendered from terminal.svg
│   └── gif/           # demo.gif — animated demo used in README/landing
├── svg-to-png.sh      # render every */svg/*.svg -> sibling */png/*.png
└── sync-assets.sh     # copy branding SVGs + og.png into packages/landing/public
```

## Regenerating

```sh
assets/svg-to-png.sh    # needs rsvg-convert (brew install librsvg)
assets/sync-assets.sh   # syncs logo.svg/og.png/favicon.ico into landing/public
```

`svg-to-png.sh` discovers SVGs recursively (`*/svg/*.svg`) and writes the
matching `*/png/` path at the SVG's intrinsic size — it covers `branding/`
and `demo/` automatically.

`sync-assets.sh` is landing-specific: it copies `branding/svg/logo.svg` to
`packages/landing/public/logo.svg`, renders `og.svg` at 1200x630 to
`public/og.png`, and rebuilds `favicon.ico` via ImageMagick when available.

## Consumers

- `README.md` — `assets/branding/png/og.png` (social preview),
  `assets/branding/png/logo.png`, `assets/demo/gif/demo.gif`
- `packages/landing/` — `public/` assets synced by `sync-assets.sh`;
  the landing page renders its own terminal demos in React
  (`src/components/molecules/Terminal.tsx`), independent of `demo/svg`
- GitHub release notes render the README's relative image paths against the
  repo, so keep referenced files committed

## Conventions

- Edit the **SVG source**, then re-run `svg-to-png.sh`. PNGs are build
  artifacts.
- `demo.svg` text must reflect **real CLI output** — commands use the
  current flags (`--file`/`--url`, not positional args) and the version
  string matches `--version` output. See `assets/AGENTS.md`.
- `demo.gif` is rendered from `demo.svg` outside this repo's scripts
  (SMIL animation → frames → ffmpeg); regenerate it when the SVG changes
  if your tooling supports it, otherwise update `terminal.svg`/`terminal.png`
  which are the static fallbacks.
