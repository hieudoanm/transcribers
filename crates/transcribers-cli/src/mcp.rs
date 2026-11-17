//! MCP server over stdio JSON-RPC.
//!
//! Tools delegate to the `transcribers` binary's own CLI so MCP behaviour
//! always matches the command line. Configure an MCP client with:
//! `transcribers mcp`

use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::process::Command;

const SERVER_INFO: &str = "transcribers-mcp";
const PROTOCOL_VERSION: &str = "2025-11-25";

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

fn annotations(read_only: bool, destructive: bool, idempotent: bool, open_world: bool) -> Value {
    json!({
        "readOnlyHint": read_only,
        "destructiveHint": destructive,
        "idempotentHint": idempotent,
        "openWorldHint": open_world
    })
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "transcribe",
                "description": "Transcribe a video or audio file to text (txt, srt, vtt or json)",
                "annotations": annotations(false, false, true, true),
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
                "annotations": annotations(true, false, true, false),
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }
            }
        ]
    })
}

/// Human-readable display title derived from the tool name
/// (`pick_meal` -> `Pick Meal`). MCP hosts show `title` in pickers.
fn titleize(name: &str) -> String {
    name.split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn with_titles(mut result: Value) -> Value {
    if let Some(tools) = result["tools"].as_array_mut() {
        for tool in tools {
            if tool.get("title").is_none() {
                if let Some(name) = tool["name"].as_str() {
                    tool["title"] = json!(titleize(name));
                }
            }
        }
    }
    result
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
        "tools/list" => with_titles(tools_list()),
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
/// Run the MCP server over streamable HTTP on `addr` until the process exits.
/// `token` (when set) requires `Authorization: Bearer <token>` on every request.
pub fn run_http(addr: &str, token: Option<String>) -> std::io::Result<()> {
    crate::mcp_http::serve(addr, token, |body: &str| {
        match serde_json::from_str::<Value>(body) {
            Ok(request) => {
                handle_request(&request).and_then(|response| serde_json::to_string(&response).ok())
            }
            Err(err) => Some(
                serde_json::to_string(&json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {"code": -32700, "message": format!("Parse error: {err}")}
                }))
                .unwrap_or_default(),
            ),
        }
    })
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_list_declares_all_tools_with_four_hints() {
        let list = tools_list();
        let tools = list["tools"].as_array().expect("tools array");
        let mut names: Vec<&str> = tools
            .iter()
            .map(|tool| tool["name"].as_str().expect("tool name"))
            .collect();
        names.sort_unstable();
        assert_eq!(names, ["list_models", "transcribe"]);
        for tool in tools {
            let name = tool["name"].as_str().expect("tool name");
            let hints = &tool["annotations"];
            for hint in [
                "readOnlyHint",
                "destructiveHint",
                "idempotentHint",
                "openWorldHint",
            ] {
                assert!(
                    hints.get(hint).is_some_and(Value::is_boolean),
                    "{name} missing boolean annotation {hint}: {hints}"
                );
            }
        }
    }

    #[test]
    fn transcribe_reaches_the_open_world() {
        let tools = tools_list()["tools"].clone();
        let transcribe = tools
            .as_array()
            .expect("tools array")
            .iter()
            .find(|tool| tool["name"] == "transcribe")
            .expect("transcribe tool");
        // URL inputs are fetched and missing models are downloaded.
        assert_eq!(transcribe["annotations"]["openWorldHint"], json!(true));
        assert_eq!(transcribe["annotations"]["readOnlyHint"], json!(false));
        assert_eq!(transcribe["annotations"]["destructiveHint"], json!(false));
        assert_eq!(transcribe["annotations"]["idempotentHint"], json!(true));

        let list_models = tools
            .as_array()
            .expect("tools array")
            .iter()
            .find(|tool| tool["name"] == "list_models")
            .expect("list_models tool");
        assert_eq!(list_models["annotations"]["readOnlyHint"], json!(true));
        assert_eq!(list_models["annotations"]["openWorldHint"], json!(false));
    }
}
