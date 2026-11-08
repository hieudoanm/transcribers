# @transcribers/landing

Static landing page for [transcribers](https://github.com/hieudoanm/transcribers) —
video and audio to text, locally. Plain HTML/CSS/JS, no build step.

Deployed to GitHub Pages at <https://hieudoanm.github.io/transcribers/>.

## Files

| File | Purpose |
| ---- | ------- |
| `index.html` | Single-page markup |
| `styles.css` | All styling — design tokens + components |
| `scripts.js` | Small progressive-enhancement script |
| `favicon.svg`, `favicon.ico` | Browser icons |
| `og-image.png` | Social share image (1200×630) |
| `robots.txt`, `sitemap.xml`, `humans.txt` | Crawler/credit metadata |
| `404.html` | GitHub Pages 404 page |

## Preview

```bash
python3 -m http.server 8000 -d packages/landing
```
