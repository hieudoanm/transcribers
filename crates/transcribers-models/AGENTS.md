# AGENTS.md — transcribers-models

Resolves whisper model names/paths and downloads missing models.

- Lib name is `models`; package is `transcribers-models`.
- Public API: `KNOWN_MODELS`, `normalize_model_name`, `model_path`.
- Model dir precedence: `$TRANSCRIBERS_MODEL_DIR` →
  `$XDG_CACHE_HOME/transcribers/models` → `$HOME/.cache/transcribers/models`.
- Names are validated against `KNOWN_MODELS` before any download; unknown
  names error with the full list. Keep `KNOWN_MODELS` in sync with the
  whisper.cpp Hugging Face repo and `normalize_model_name` semantics
  (`ggml-` prefix / `.bin` suffix stripping).
- Downloads use `curl -fSL -C -` to a `.part` file, then verify
  size > 0 and `ggml` magic before rename — do not skip validation.
- No dependencies; errors are `String`s.
