// The terminal-style REPL console panel's Tauri substrate: starts the real
// `my-lisp` CLI binary (see `repl_process::resolve_my_lisp_binary`) as a
// live child process and forwards every stdout/stderr line to the frontend
// as a `repl-console-output` event, mirroring how `build_runner.rs` streams
// `ProcessEvent`s for the Build Output panel.

use crate::repl_process::{my_idea_repo_root, resolve_my_lisp_binary, ReplProcess, ReplProcessLine, ReplProcessStream};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

pub const REPL_CONSOLE_EVENT: &str = "repl-console-output";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplConsoleLineEvent {
    pub stream: &'static str,
    pub line: String,
}

impl From<ReplProcessLine> for ReplConsoleLineEvent {
    fn from(line: ReplProcessLine) -> Self {
        Self {
            stream: match line.stream {
                ReplProcessStream::Stdout => "stdout",
                ReplProcessStream::Stderr => "stderr",
            },
            line: line.line,
        }
    }
}

/// Holds the single live console process, if one has been started. Lazily
/// started (not at app boot) since building the submodule binary can take a
/// moment the first time.
#[derive(Default)]
pub struct ManagedReplConsole(Mutex<Option<ReplProcess>>);

/// Starts the real `my-lisp` REPL as a child process for the console panel,
/// if it isn't already running. Idempotent: a second call while one is
/// already live is a no-op.
#[tauri::command]
pub fn start_repl_console(app: AppHandle, console: State<'_, ManagedReplConsole>) -> Result<(), String> {
    let mut guard = console
        .0
        .lock()
        .map_err(|_| "repl console state poisoned".to_string())?;
    if guard.is_some() {
        return Ok(());
    }

    let binary = resolve_my_lisp_binary(&my_idea_repo_root())?;
    let app_handle = app.clone();
    let process = ReplProcess::spawn(&binary, &[], move |line| {
        let _ = app_handle.emit(REPL_CONSOLE_EVENT, ReplConsoleLineEvent::from(line));
    })?;
    *guard = Some(process);
    Ok(())
}

/// Sends one line to the live console process's stdin, exactly as if it had
/// been typed into the real REPL's own terminal.
#[tauri::command]
pub fn send_repl_console_line(line: String, console: State<'_, ManagedReplConsole>) -> Result<(), String> {
    let guard = console
        .0
        .lock()
        .map_err(|_| "repl console state poisoned".to_string())?;
    match guard.as_ref() {
        Some(process) => process.write_line(&line),
        None => Err("repl console is not running yet — call start_repl_console first".to_string()),
    }
}
