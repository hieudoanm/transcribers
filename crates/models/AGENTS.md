# AGENTS.md — models

Resolves whisper model names/paths and downloads missing models.

- Public API: `model_path(model: &str) -> Result<PathBuf, String>`
- Cache dir: `$TRANSCRIBERS_MODEL_DIR`, else `$XDG_CACHE_HOME/transcribers`
  or `$HOME/.cache/transcribers`; models live under `models/ggml-<name>.bin`.
- Download via `curl -fSL` to a `.part` file, then rename into place.
- Keep name normalisation (`ggml-` prefix / `.bin` suffix stripping) in sync
  with the names published on the whisper.cpp Hugging Face repo.
- No dependencies; errors are `String`s.
