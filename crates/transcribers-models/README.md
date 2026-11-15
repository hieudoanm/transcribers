# transcribers-models

Whisper model resolution, validation, and download for `transcribers`.

Lib name is `models` (package `transcribers-models`).

## API

```rust
pub const KNOWN_MODELS: &[&str];   // canonical model names

pub fn normalize_model_name(model: &str) -> String;
pub fn model_path(model: &str) -> Result<PathBuf, String>;
```

## Resolution order (`model_path`)

1. Existing filesystem path → returned unchanged (custom `.bin` models).
2. Path-like input (`*.bin`, contains `/` or `\`) that doesn't exist → error.
3. Name lookup: `normalize_model_name` strips `ggml-` prefix / `.bin`
   suffix; the result must be in `KNOWN_MODELS` or an error lists the
   valid names.
4. Look for `ggml-<name>.bin` in the model dir; return it if present.
5. Otherwise download from
   `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-<name>.bin`
   with `curl -fSL -C -` (resumes partial downloads) into a `.part` file,
   verify size > 0 and `ggml` magic bytes, then rename into place.

## Model directory

`$TRANSCRIBERS_MODEL_DIR` when set (points directly at the directory of
`.bin` files), else `$XDG_CACHE_HOME/transcribers/models/`, else
`$HOME/.cache/transcribers/models/`.

## `KNOWN_MODELS`

Canonical names published by whisper.cpp (`tiny`, `base`, `small`,
`medium`, `large`, `large-v3`, `large-v3-turbo`, and `.en` variants).
`--list-models` prints this list. Keep it in sync with the Hugging Face
repo; a test asserts every entry normalises correctly.

## Requirements / notes

- `curl` on `PATH` for downloads only; cached/local models need nothing.
- Failed downloads remove the `.part` file; corrupt downloads (empty or
  non-`ggml` magic) are rejected before rename.
- Zero dependencies; errors are `String`s.
- Unit tests cover `normalize_model_name` and `KNOWN_MODELS` consistency.
