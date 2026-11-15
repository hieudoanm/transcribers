//! MCP server over stdio JSON-RPC.
//!
//! Tools delegate to the `transcribers` binary's own CLI so MCP behaviour
//! always matches the command line. Configure an MCP client with:
//! `transcribers mcp`

use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::process::Command;

const SERVER_INFO: &str = "transcribers-mcp";
const PROTOCOL_VERSION: &str = "2024-11-05";

fn arg(params: &Value, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|s| !s.is_empty())
}

fn num(params: &Value, key: &str) -> Option<String> {
    params.get(key).and_then(|v| {
        v.as_u64()
            .map(|n| n.to_string())
            .or_else(|| v.as_i64().map(|n| n.to_string()))
            .or_else(|| v.as_f64().map(|n| n.to_string()))
    })
}

fn run_cli(args: &[String]) -> Value {
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(err) => {
            return json!({
                "content": [{"type": "text", "text": format!("cannot locate transcribers binary: {err}")}],
                "isError": true
            })
        }
    };
    match Command::new(exe).args(args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if output.status.success() {
                let text = if stdout.is_empty() { stderr } else { stdout };
                json!({"content": [{"type": "text", "text": text}]})
            } else {
                json!({
                    "content": [{"type": "text", "text": stderr}],
                    "isError": true
                })
            }
        }
        Err(err) => json!({
            "content": [{"type": "text", "text": format!("failed to run transcribers: {err}")}],
            "isError": true
        }),
    }
}

fn build_args(name: &str, params: &Value) -> Option<Vec<String>> {
    match name {
        "list_models" => Some(vec!["--list-models".to_string()]),
        "transcribe" => {
            let input = arg(params, "input")?;
            let flag = if input.starts_with("http://") || input.starts_with("https://") {
                "--url"
            } else {
                "--file"
            };
            let mut args = vec![flag.to_string(), input];
            if let Some(v) = arg(params, "model") {
                args.extend(["--model".to_string(), v]);
            }
            if let Some(v) = arg(params, "language") {
                args.extend(["--language".to_string(), v]);
            }
            if let Some(v) = arg(params, "format") {
                args.extend(["--format".to_string(), v]);
            }
            if let Some(v) = arg(params, "output") {
                args.extend(["--output".to_string(), v]);
            }
            if let Some(v) = num(params, "threads") {
                args.extend(["--threads".to_string(), v]);
            }
            if let Some(v) = num(params, "start") {
                args.extend(["--start".to_string(), v]);
            }
            if let Some(v) = num(params, "duration") {
                args.extend(["--duration".to_string(), v]);
            }
            if let Some(v) = arg(params, "prompt") {
                args.extend(["--prompt".to_string(), v]);
            }
            if params.get("translate").and_then(Value::as_bool) == Some(true) {
                args.push("--translate".to_string());
            }
            Some(args)
        }
        _ => None,
    }
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "transcribe",
                "description": "Transcribe a video or audio file to text (txt, srt, vtt or json)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "input": {"type": "string", "description": "Path to the video or audio file, or an http(s) URL (YouTube, Instagram, ...)"},
                        "model": {"type": "string", "description": "Model name (tiny, base, small, medium, large, base.en, ...) or path to a .bin file. Default: base.en"},
                        "language": {"type": "string", "description": "Spoken language code or 'auto'. Default: auto"},
                        "format": {"type": "string", "description": "Output format: txt, srt, vtt, json. Default: srt"},
                        "output": {"type": "string", "description": "Write output to PATH instead of stdout"},
                        "threads": {"type": "integer", "description": "Decoder threads. Default: auto"},
                        "start": {"type": "number", "description": "Start transcription at SEC seconds into the media"},
                        "duration": {"type": "number", "description": "Transcribe at most SEC seconds"},
                        "prompt": {"type": "string", "description": "Initial prompt to bias recognition"},
                        "translate": {"type": "boolean", "description": "Translate speech to English"}
                    },
                    "required": ["input"],
                    "additionalProperties": false
                }
            },
            {
                "name": "list_models",
                "description": "List known whisper model names",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }
            }
        ]
    })
}

fn handle_request(request: &Value) -> Option<Value> {
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let has_id = request.get("id").is_some();

    let result = match method {
        "initialize" => json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": SERVER_INFO, "version": env!("CARGO_PKG_VERSION")}
        }),
        "ping" => Value::Null,
        "tools/list" => tools_list(),
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or(Value::Null);
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            match build_args(name, &arguments) {
                Some(args) => run_cli(&args),
                None => json!({
                    "content": [{"type": "text", "text": format!("Unknown tool: {name}")}],
                    "isError": true
                }),
            }
        }
        _ => {
            if !has_id {
                return None;
            }
            return Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("Method not found: {method}")}
            }));
        }
    };

    if !has_id {
        return None;
    }
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

/// Run the MCP server on stdin/stdout until EOF.
pub fn run() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(trimmed) {
            Ok(request) => {
                if let Some(response) = handle_request(&request) {
                    writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                    stdout.flush()?;
                }
            }
            Err(err) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {"code": -32700, "message": format!("Parse error: {err}")}
                });
                writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                stdout.flush()?;
            }
        }
    }
    Ok(())
}
