use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const MODEL_BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

/// Model names published on the whisper.cpp Hugging Face repo
/// (`ggml-<name>.bin`), plus the `large` alias for `large-v3`.
pub const KNOWN_MODELS: &[&str] = &[
    "tiny",
    "tiny.en",
    "base",
    "base.en",
    "small",
    "small.en",
    "medium",
    "medium.en",
    "large",
    "large-v1",
    "large-v2",
    "large-v3",
    "large-v3-turbo",
    "distil-large-v2",
    "distil-large-v3",
    "distil-medium.en",
    "distil-small.en",
];

/// Normalises a user-supplied model name to the bare whisper.cpp name:
/// strips a leading `ggml-` and a trailing `.bin`, maps `large` to
/// `large-v3`, and lowercases.
pub fn normalize_model_name(model: &str) -> String {
    let name = model
        .strip_prefix("ggml-")
        .unwrap_or(model)
        .trim_end_matches(".bin")
        .to_ascii_lowercase();
    if name == "large" {
        "large-v3".to_string()
    } else {
        name
    }
}

/// Resolves a model name (e.g. `base.en`, `small`) to a local `.bin` path,
/// downloading it into the cache directory on first use. A path to an
/// existing file is returned as-is.
pub fn model_path(model: &str) -> Result<PathBuf, String> {
    let path = Path::new(model);
    if path.exists() {
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
        return Err(format!("model path is not a file: {}", path.display()));
    }
    if model.ends_with(".bin") || model.contains('/') || model.contains('\\') {
        return Err(format!("model file not found: {}", path.display()));
    }

    let name = normalize_model_name(model);
    if !KNOWN_MODELS.contains(&name.as_str()) {
        return Err(format!(
            "unknown model '{model}'; known models: {} (or pass a path to a local .bin file)",
            KNOWN_MODELS.join(", ")
        ));
    }
    let file_name = format!("ggml-{name}.bin");
    // TRANSCRIBERS_MODEL_DIR points directly at the directory holding the
    // .bin files, bypassing the cache-dir convention entirely.
    let cache_dir = env::var_os("TRANSCRIBERS_MODEL_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| cache_dir().join("models"));
    let dest = cache_dir.join(&file_name);
    if dest.exists() {
        return Ok(dest);
    }

    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("cannot create {}: {e}", cache_dir.display()))?;
    eprintln!("downloading model {file_name} ...");
    let url = format!("{MODEL_BASE_URL}/{file_name}");
    let tmp = cache_dir.join(format!("{file_name}.part"));
    // `-C -` resumes a partial download left by an interrupted run.
    let status = Command::new("curl")
        .args(["-fSL", "-C", "-", "-o"])
        .arg(&tmp)
        .arg(&url)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("failed to run curl: {e}"))?;
    if !status.success() {
        let _ = fs::remove_file(&tmp);
        return Err(format!(
            "failed to download {url}; pass a full path to a local .bin model instead"
        ));
    }
    let size = fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0);
    if size == 0 {
        let _ = fs::remove_file(&tmp);
        return Err(format!("downloaded model is empty: {url}"));
    }
    fs::rename(&tmp, &dest).map_err(|e| format!("cannot move downloaded model: {e}"))?;
    Ok(dest)
}

fn cache_dir() -> PathBuf {
    if let Some(dir) = env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(dir).join("transcribers");
    }
    #[cfg(windows)]
    {
        if let Some(dir) = env::var_os("LOCALAPPDATA") {
            return PathBuf::from(dir).join("transcribers");
        }
        if let Some(home) = env::var_os("USERPROFILE") {
            return PathBuf::from(home).join(".cache").join("transcribers");
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(".cache").join("transcribers");
        }
    }
    PathBuf::from(".").join("transcribers")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_prefix_and_suffix() {
        assert_eq!(normalize_model_name("ggml-base.en.bin"), "base.en");
        assert_eq!(normalize_model_name("BASE.EN"), "base.en");
        assert_eq!(normalize_model_name("small"), "small");
        assert_eq!(normalize_model_name("large"), "large-v3");
    }

    #[test]
    fn known_models_are_listed() {
        for m in [
            "tiny",
            "base.en",
            "small",
            "medium",
            "large-v3",
            "large-v3-turbo",
        ] {
            assert!(KNOWN_MODELS.contains(&m), "missing {m}");
        }
    }

    #[test]
    fn unknown_name_fails_before_download() {
        // No path separators, no .bin suffix -> treated as a model name and
        // rejected against the allowlist without touching the network.
        let err = model_path("definitely-not-a-model").unwrap_err();
        assert!(err.contains("unknown model"), "{err}");
    }

    #[test]
    fn missing_path_with_bin_suffix_errors() {
        let err = model_path("/nonexistent/model.bin").unwrap_err();
        assert!(err.contains("not found"), "{err}");
    }
}
