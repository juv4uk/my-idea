// The terminal-style REPL console panel's Tauri substrate: starts the real
// `my-lisp` CLI binary as a live process and forwards every stdout/stderr
// line to the frontend as a `repl-console-output` event, mirroring how
// `build_runner.rs` streams `ProcessEvent`s for the Build Output panel.
//
// Two ways the binary gets there, tried in order:
//
// 1. **Sidecar** (packaged releases): `.github/workflows/publish-release.yml`
//    builds `my-lisp` fresh from its own latest `main` for every target
//    platform and bundles it via Tauri's `externalBin` mechanism — an
//    installed app never needs git, cargo, or a network connection to run
//    the console at all.
// 2. **Local build** (source/dev checkouts, or an install that predates
//    sidecar bundling for its platform): `repl_process::
//    resolve_or_fetch_my_lisp_binary` — the local `external/my-lisp`
//    submodule if present, or a fresh clone of its pinned commit otherwise.

use crate::repl_process::{my_idea_repo_root, resolve_or_fetch_my_lisp_binary, ReplProcess, ReplProcessLine, ReplProcessStream};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

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

/// Either the bundled sidecar or a locally spawned process — both drive the
/// exact same real `my-lisp` REPL binary, just launched through different
/// Tauri machinery depending on how it was resolved.
enum ConsoleProcess {
    Sidecar(Mutex<tauri_plugin_shell::process::CommandChild>),
    Local(ReplProcess),
}

impl ConsoleProcess {
    fn write_line(&self, line: &str) -> Result<(), String> {
        match self {
            ConsoleProcess::Local(process) => process.write_line(line),
            ConsoleProcess::Sidecar(child) => {
                let mut child = child
                    .lock()
                    .map_err(|_| "repl sidecar stdin poisoned".to_string())?;
                let mut bytes = line.as_bytes().to_vec();
                bytes.push(b'\n');
                child
                    .write(&bytes)
                    .map_err(|error| format!("failed to write to repl sidecar: {error}"))
            }
        }
    }
}

/// Holds the single live console process, if one has been started. Lazily
/// started (not at app boot) since a local fallback build can take a moment
/// the first time.
#[derive(Default)]
pub struct ManagedReplConsole(Mutex<Option<ConsoleProcess>>);

/// Starts the real `my-lisp` REPL for the console panel, if it isn't
/// already running. Idempotent: a second call while one is already live is
/// a no-op.
#[tauri::command]
pub fn start_repl_console(app: AppHandle, console: State<'_, ManagedReplConsole>) -> Result<(), String> {
    let mut guard = console
        .0
        .lock()
        .map_err(|_| "repl console state poisoned".to_string())?;
    if guard.is_some() {
        return Ok(());
    }

    if let Some(process) = try_start_sidecar(&app) {
        *guard = Some(process);
        return Ok(());
    }

    // No bundled sidecar for this platform/build — fall back to a local
    // submodule checkout (dev) or fetching the pinned commit from GitHub.
    let repo_root = my_idea_repo_root();
    if !repo_root.join("external").join("my-lisp").join("Cargo.toml").exists() {
        let _ = app.emit(
            REPL_CONSOLE_EVENT,
            ReplConsoleLineEvent {
                stream: "system",
                line: "No bundled my-lisp sidecar found; building from source (first run only, needs network + git + cargo)…"
                    .to_string(),
            },
        );
    }
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| format!("could not resolve the app cache directory: {error}"))?;
    let binary = resolve_or_fetch_my_lisp_binary(&repo_root, &cache_dir)?;

    let app_handle = app.clone();
    let process = ReplProcess::spawn(&binary, &[], move |line| {
        let _ = app_handle.emit(REPL_CONSOLE_EVENT, ReplConsoleLineEvent::from(line));
    })?;
    *guard = Some(ConsoleProcess::Local(process));
    Ok(())
}

/// Tries the bundled sidecar; returns `None` on any failure (no sidecar for
/// this platform, scope not configured, etc.) so the caller can fall back.
fn try_start_sidecar(app: &AppHandle) -> Option<ConsoleProcess> {
    let command = app.shell().sidecar("my-lisp").ok()?;
    let (mut rx, child) = command.spawn().ok()?;

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            let line_event = match event {
                CommandEvent::Stdout(bytes) => Some(ReplConsoleLineEvent {
                    stream: "stdout",
                    line: String::from_utf8_lossy(&bytes).trim_end().to_string(),
                }),
                CommandEvent::Stderr(bytes) => Some(ReplConsoleLineEvent {
                    stream: "stderr",
                    line: String::from_utf8_lossy(&bytes).trim_end().to_string(),
                }),
                CommandEvent::Error(message) => Some(ReplConsoleLineEvent {
                    stream: "stderr",
                    line: message,
                }),
                _ => None,
            };
            if let Some(line_event) = line_event {
                let _ = app_handle.emit(REPL_CONSOLE_EVENT, line_event);
            }
        }
    });

    Some(ConsoleProcess::Sidecar(Mutex::new(child)))
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
