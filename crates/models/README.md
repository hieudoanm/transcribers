# models

Whisper model resolution and download for `transcribers`.

## API

```rust
pub fn model_path(model: &str) -> Result<PathBuf, String>
```

Resolution order:

1. If `model` is an existing filesystem path, return it unchanged.
2. If it looks like a path (`*.bin`, contains `/` or `\`) but does not exist,
   return an error.
3. Otherwise treat it as a model name: strip a leading `ggml-` and trailing
   `.bin`, then look for `ggml-<name>.bin` in the cache directory.
4. If missing, download it from
   `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-<name>.bin`
   using `curl` into `<cache>/models/`, via a `.part` temp file.

## Cache location

`$XDG_CACHE_HOME/transcribers/models/` when `XDG_CACHE_HOME` is set,
otherwise `$HOME/.cache/transcribers/models/`.

## Requirements

`curl` on `PATH` for downloads (not needed when the model is already cached).

## Notes

- Zero dependencies.
- Model names such as `base.en`, `small`, `large-v3` map to
  `ggml-base.en.bin`, `ggml-small.bin`, `ggml-large-v3.bin`.
- A failed download removes the `.part` file and returns an error suggesting
  a local `.bin` path instead.
