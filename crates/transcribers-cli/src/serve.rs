//! `transcribers serve` — HTTP API for transcription.
//!
//! `POST /transcribe` accepts `multipart/form-data` with either a `file`
//! field (media upload) or a `url` field (http(s) URL downloaded via
//! yt-dlp). Optional text fields mirror the CLI flags: `model`, `language`,
//! `format`, `threads`, `start`, `duration`, `prompt`, `translate`.
//! The response body is the rendered transcript; the content type matches
//! the format (`application/json` for `json`, `text/plain` otherwise).
//!
//! `GET /` serves Swagger UI backed by `assets/openapi.{yaml,json}`;
//! `GET /health` returns `ok`. Bind address defaults to 127.0.0.1:8080.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use tokio::io::AsyncWriteExt;
use whisper::Format;

use crate::{default_threads, download_media, is_url, parse_seconds, transcribe_text, Job};

#[derive(Parser)]
#[command(
    name = "transcribers",
    bin_name = "transcribers serve",
    about = "Run an HTTP server that transcribes uploaded files or URLs"
)]
struct ServeArgs {
    /// Address to bind.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on.
    #[arg(long, default_value_t = 8080)]
    port: u16,
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn work_dir() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("transcribers-serve-{}-{n}", std::process::id()))
}

fn err_response(status: StatusCode, msg: impl Into<String>) -> Response {
    (status, Json(serde_json::json!({"error": msg.into()}))).into_response()
}

async fn health() -> &'static str {
    "ok"
}

const OPENAPI_YAML: &str = include_str!("../assets/openapi.yaml");
const OPENAPI_JSON: &str = include_str!("../assets/openapi.json");

const SWAGGER_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>transcribers API</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
  <div id="ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    window.onload = () => SwaggerUIBundle({ url: "/openapi.json", dom_id: "#ui" });
  </script>
</body>
</html>"##;

async fn index() -> Html<&'static str> {
    Html(SWAGGER_HTML)
}

async fn openapi_json() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/json")], OPENAPI_JSON)
}

async fn openapi_yaml() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/yaml")], OPENAPI_YAML)
}

/// Streams the uploaded `file` field to `dest`.
async fn save_upload(
    field: &mut axum::extract::multipart::Field<'_>,
    dest: &Path,
) -> Result<(), String> {
    let mut out = tokio::fs::File::create(dest)
        .await
        .map_err(|e| format!("cannot create {}: {e}", dest.display()))?;
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|e| format!("failed reading upload: {e}"))?
    {
        out.write_all(&chunk)
            .await
            .map_err(|e| format!("failed writing {}: {e}", dest.display()))?;
    }
    Ok(())
}

fn field<'a>(fields: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
        .filter(|v| !v.is_empty())
}

fn parse_num<T: std::str::FromStr>(
    fields: &[(String, String)],
    key: &str,
) -> Result<Option<T>, String>
where
    T::Err: std::fmt::Display,
{
    match field(fields, key) {
        Some(v) => v
            .parse::<T>()
            .map(Some)
            .map_err(|e| format!("'{key}' must be a number, got '{v}': {e}")),
        None => Ok(None),
    }
}

fn parse_flag(fields: &[(String, String)], key: &str) -> bool {
    field(fields, key).is_some_and(|v| matches!(v, "1" | "true" | "yes" | "on"))
}

async fn transcribe_handler(mut mp: Multipart) -> Response {
    let work = work_dir();
    if let Err(e) = std::fs::create_dir_all(&work) {
        return err_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    let mut file: Option<PathBuf> = None;
    let mut url: Option<String> = None;
    let mut fields: Vec<(String, String)> = Vec::new();

    loop {
        let mut field = match mp.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => {
                let _ = std::fs::remove_dir_all(&work);
                return err_response(StatusCode::BAD_REQUEST, format!("bad multipart body: {e}"));
            }
        };
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                let fname = field.file_name().unwrap_or("upload").to_string();
                let ext = Path::new(&fname)
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy()))
                    .unwrap_or_default();
                let dest = work.join(format!("upload{ext}"));
                if let Err(e) = save_upload(&mut field, &dest).await {
                    let _ = std::fs::remove_dir_all(&work);
                    return err_response(StatusCode::BAD_REQUEST, e);
                }
                file = Some(dest);
            }
            "url" => match field.text().await {
                Ok(v) => url = Some(v),
                Err(e) => {
                    let _ = std::fs::remove_dir_all(&work);
                    return err_response(StatusCode::BAD_REQUEST, format!("bad 'url' field: {e}"));
                }
            },
            other => {
                if let Ok(v) = field.text().await {
                    fields.push((other.to_string(), v));
                }
            }
        }
    }

    macro_rules! bail {
        ($e:expr) => {{
            let _ = std::fs::remove_dir_all(&work);
            return err_response(StatusCode::BAD_REQUEST, $e);
        }};
    }

    if file.is_none() && url.as_deref().is_none_or(str::is_empty) {
        bail!("missing input: provide a 'file' upload or a 'url' field");
    }
    if let Some(u) = url.as_deref().filter(|u| !u.is_empty()) {
        if !is_url(u) {
            bail!(format!("'url' must be an http(s) URL, got '{u}'"));
        }
    }

    let format = match field(&fields, "format") {
        Some(f) => match Format::parse(f) {
            Ok(fmt) => fmt,
            Err(e) => bail!(e),
        },
        None => Format::Srt,
    };
    let threads: Option<usize> = match parse_num(&fields, "threads") {
        Ok(t) => t,
        Err(e) => bail!(e),
    };
    if threads == Some(0) {
        bail!("'threads' must be >= 1");
    }
    let start = match field(&fields, "start") {
        Some(v) => match parse_seconds(v) {
            Ok(s) => Some(s),
            Err(e) => bail!(e),
        },
        None => None,
    };
    let duration = match field(&fields, "duration") {
        Some(v) => match parse_seconds(v) {
            Ok(d) if d > 0.0 => Some(d),
            _ => bail!("'duration' must be > 0"),
        },
        None => None,
    };

    let job = Job {
        model: field(&fields, "model").unwrap_or("base.en").to_string(),
        language: field(&fields, "language").unwrap_or("auto").to_string(),
        format,
        threads: threads.unwrap_or_else(default_threads),
        start,
        duration,
        prompt: field(&fields, "prompt").map(str::to_string),
        translate: parse_flag(&fields, "translate"),
    };

    let work2 = work.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let media = match url.filter(|u| !u.is_empty()) {
            Some(u) => download_media(&u, &work2)?,
            None => file.ok_or_else(|| "missing input".to_string())?,
        };
        transcribe_text(&media, &work2, &job).map(|(text, _)| text)
    })
    .await;

    let _ = std::fs::remove_dir_all(&work);

    match result {
        Ok(Ok(text)) => {
            let content_type = if format == Format::Json {
                "application/json"
            } else {
                "text/plain; charset=utf-8"
            };
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .body(Body::from(text))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Ok(Err(e)) => err_response(StatusCode::INTERNAL_SERVER_ERROR, e),
        Err(e) => err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("transcription task failed: {e}"),
        ),
    }
}

/// Parse `transcribers serve [--host H] [--port P]` and run the server.
pub fn run() -> Result<(), String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    // argv[0] is "serve"; clap expects a binary name first.
    let args = ServeArgs::try_parse_from(
        std::iter::once("transcribers".to_string()).chain(argv.into_iter().skip(1)),
    )
    .unwrap_or_else(|e| e.exit());

    let rt =
        tokio::runtime::Runtime::new().map_err(|e| format!("cannot start async runtime: {e}"))?;
    rt.block_on(async move {
        let app = Router::new()
            .route("/", get(index))
            .route("/health", get(health))
            .route("/openapi.json", get(openapi_json))
            .route("/openapi.yaml", get(openapi_yaml))
            .route("/transcribe", post(transcribe_handler))
            .layer(DefaultBodyLimit::disable());
        let addr = format!("{}:{}", args.host, args.port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("cannot bind {addr}: {e}"))?;
        eprintln!("serving on http://{addr}");
        eprintln!("POST /transcribe with 'file' or 'url' (multipart/form-data)");
        axum::serve(listener, app)
            .await
            .map_err(|e| format!("server error: {e}"))
    })
}
