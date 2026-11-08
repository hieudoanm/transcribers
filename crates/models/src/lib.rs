use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const MODEL_BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

/// Resolves a model name (e.g. `base.en`, `small`) to a local `.bin` path,
/// downloading it into the cache directory on first use. A path to an
/// existing file is returned as-is.
pub fn model_path(model: &str) -> Result<PathBuf, String> {
    let path = Path::new(model);
    if path.exists() {
        return Ok(path.to_path_buf());
    }
    if model.ends_with(".bin") || model.contains('/') || model.contains('\\') {
        return Err(format!("model file not found: {}", path.display()));
    }

    let name = model
        .strip_prefix("ggml-")
        .unwrap_or(model)
        .trim_end_matches(".bin");
    let file_name = format!("ggml-{name}.bin");
    let cache_dir = cache_dir().join("models");
    let dest = cache_dir.join(&file_name);
    if dest.exists() {
        return Ok(dest);
    }

    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("cannot create {}: {e}", cache_dir.display()))?;
    eprintln!("downloading model {file_name} ...");
    let url = format!("{MODEL_BASE_URL}/{file_name}");
    let tmp = cache_dir.join(format!("{file_name}.part"));
    let status = Command::new("curl")
        .args(["-fSL", "-o"])
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
    fs::rename(&tmp, &dest).map_err(|e| format!("cannot move downloaded model: {e}"))?;
    Ok(dest)
}

fn cache_dir() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("transcribers")
}
