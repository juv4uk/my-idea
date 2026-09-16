//! Test-only harness for #50 (IDE-LIVE-EDITOR-WITNESS-1).
//!
//! Exposes the exact same five operations `lib.rs`'s real Tauri commands
//! (`editor_dispatch_key`/`editor_invoke_command`/`editor_emit_event`/
//! `editor_list_commands`/`editor_list_keymaps`) wrap, over the real
//! production `ManagedReplSession`/`EditorCommandRegistry` — the identical
//! Rust types and command names the shipped app uses, no test double.
//!
//! What is NOT real here: the transport. A packaged my-idea talks to this
//! logic over Tauri's own WebView IPC; automating that would need
//! `tauri-driver` + a WebDriver-capable WebView (`webkit2gtk-driver` on
//! Linux), which this environment cannot install (no root, and it isn't
//! packaged in this Guix channel either — a documented, not silently
//! ignored, gap). This harness swaps only that transport for
//! newline-delimited JSON over stdio, so `tests/live_editor_witness.mjs`
//! can drive it from a real headless-browser CodeMirror instance via
//! Playwright's `page.exposeFunction`. Same contract, same Rust logic,
//! same real my-lisp evaluation — a different wire.
//!
//! Protocol: one JSON object per line on stdin: `{"cmd": "...", "args": {...}}`.
//! One JSON object per line on stdout: `{"ok": true, "result": ...}` or
//! `{"ok": false, "error": "..."}`.

use my_idea_lib::editor_api::EditorState;
use my_idea_lib::ManagedReplSession;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn state_from(args: &Value) -> Result<EditorState, String> {
    serde_json::from_value(
        args.get("state")
            .cloned()
            .ok_or_else(|| "missing \"state\"".to_string())?,
    )
    .map_err(|error| error.to_string())
}

fn string_arg(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("missing \"{key}\""))
}

fn handle(
    session: &ManagedReplSession,
    plugins_dir: &std::path::Path,
    cmd: &str,
    args: &Value,
) -> Result<Value, String> {
    match cmd {
        "reload_plugins" => {
            let report = session.load_plugins(plugins_dir);
            Ok(json!({
                "loaded": report.loaded.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                "failures": report.failures.iter().map(|f| f.message.clone()).collect::<Vec<_>>(),
            }))
        }
        "editor_dispatch_key" => {
            let key = string_arg(args, "key")?;
            let state = state_from(args)?;
            let effect = session.dispatch_key(&key, state)?;
            Ok(json!(effect))
        }
        "editor_invoke_command" => {
            let name = string_arg(args, "name")?;
            let state = state_from(args)?;
            let effect = session.invoke_command(&name, state)?;
            Ok(json!(effect))
        }
        "editor_emit_event" => {
            let event = string_arg(args, "event")?;
            let state = state_from(args)?;
            let (mut applied, mut failures) = (Vec::new(), Vec::new());
            for result in session.emit_event(&event, state) {
                match result {
                    Ok(effect) => applied.push(effect),
                    Err(message) => failures.push(message),
                }
            }
            Ok(json!({"applied": applied, "failures": failures}))
        }
        "editor_list_commands" => Ok(json!(session.list_commands())),
        "editor_list_keymaps" => Ok(json!(session
            .list_keymaps()
            .into_iter()
            .map(|(key, command)| json!({"key": key, "command": command}))
            .collect::<Vec<_>>())),
        other => Err(format!("unknown harness command: {other}")),
    }
}

fn main() {
    let plugins_dir = std::path::PathBuf::from(
        std::env::args().nth(1).expect("usage: editor_bridge_harness <plugins-dir>"),
    );
    let session = ManagedReplSession::new();
    let report = session.load_plugins(&plugins_dir);
    for failure in &report.failures {
        eprintln!("plugin load failure: {}: {}", failure.path.display(), failure.message);
    }
    eprintln!("editor_bridge_harness ready");

    let stdin = io::stdin();
    let stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line.expect("stdin read should succeed");
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => {
                let cmd = request.get("cmd").and_then(Value::as_str).unwrap_or_default();
                let empty = json!({});
                let args = request.get("args").unwrap_or(&empty);
                match handle(&session, &plugins_dir, cmd, args) {
                    Ok(result) => json!({"ok": true, "result": result}),
                    Err(error) => json!({"ok": false, "error": error}),
                }
            }
            Err(error) => json!({"ok": false, "error": format!("invalid request JSON: {error}")}),
        };
        let mut out = stdout.lock();
        writeln!(out, "{response}").expect("stdout write should succeed");
        out.flush().expect("stdout flush should succeed");
    }
}
