use std::path::{Path, PathBuf};
use std::sync::Mutex;
use my_lisp_literate::SourceMode;
use crate::editor_api::{EditorCommandRegistry, EditorEffect, EditorState};
use crate::LispEvaluation;

/// Desired workspace target upon starting the desktop application.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StartupTarget {
    DefaultWorkspace,
    Path(PathBuf),
}

/// Parses CLI arguments to resolve the startup target workspace or file.
pub fn parse_startup_target(args: impl IntoIterator<Item = String>) -> Result<StartupTarget, String> {
    let mut items = args.into_iter();
    match (items.next(), items.next()) {
        (None, _) => Ok(StartupTarget::DefaultWorkspace),
        (Some(path), None) => Ok(StartupTarget::Path(PathBuf::from(path))),
        (Some(_), Some(_)) => Err("at most one startup path is supported".to_owned()),
    }
}

/// Resolves an initial workspace root from the given startup target.
pub fn resolve_initial_workspace(target: StartupTarget) -> Option<PathBuf> {
    match target {
        StartupTarget::Path(path) => {
            let canonical = path.canonicalize().unwrap_or(path);
            if canonical.is_dir() {
                Some(canonical)
            } else if let Some(parent) = canonical.parent() {
                Some(parent.to_path_buf())
            } else {
                Some(canonical)
            }
        }
        StartupTarget::DefaultWorkspace => None,
    }
}

/// Retained `my-lisp` session preserving definitions across sequential evaluations.
pub struct ReplSession {
    session: my_lisp::Session,
}

impl Default for ReplSession {
    fn default() -> Self {
        Self {
            session: my_lisp::Session::default(),
        }
    }
}

impl ReplSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate(&mut self, source: &str) -> Result<LispEvaluation, String> {
        self.evaluate_mode(source, SourceMode::PureLisp)
    }

    /// Дає доступ до внутрішнього оточення my-lisp — потрібно лише хостовим
    /// розширенням у цьому крейті (наприклад, `editor_api`), які реєструють
    /// власні capability-функції та токени поверх звичайного REPL.
    ///
    /// Exposes the underlying my-lisp environment — needed only by in-crate
    /// host extensions (e.g. `editor_api`) that install their own capability
    /// functions and tokens on top of the ordinary REPL.
    pub(crate) fn environment(&self) -> &my_lisp::Environment {
        &self.session.environment
    }

    pub fn evaluate_mode(&mut self, source: &str, mode: SourceMode) -> Result<LispEvaluation, String> {
        let (result, forms) = my_lisp_literate::eval_literate(source, mode, &mut self.session)
            .map_err(|error| error.to_string())?;

        Ok(LispEvaluation {
            value: result.value.to_string(),
            output: result.output,
            ast: format!("{forms:#?}"),
            engine: "my-lisp · Rust",
        })
    }
}

enum ReplCommand {
    Evaluate {
        source: String,
        mode: Option<String>,
        reply: std::sync::mpsc::Sender<Result<LispEvaluation, String>>,
    },
    LoadPlugins {
        config_dir: PathBuf,
        reply: std::sync::mpsc::Sender<crate::plugins::PluginLoadReport>,
    },
    DispatchKey {
        key: String,
        state: EditorState,
        reply: std::sync::mpsc::Sender<Result<EditorEffect, String>>,
    },
    InvokeCommand {
        name: String,
        state: EditorState,
        reply: std::sync::mpsc::Sender<Result<EditorEffect, String>>,
    },
    EmitEvent {
        event: String,
        state: EditorState,
        reply: std::sync::mpsc::Sender<Vec<Result<EditorEffect, String>>>,
    },
    ListCommands {
        reply: std::sync::mpsc::Sender<Vec<String>>,
    },
    ListKeymaps {
        reply: std::sync::mpsc::Sender<Vec<(String, String)>>,
    },
}

/// Managed wrapper for `ReplSession` to be stored in Tauri state.
/// Hosts the single-threaded `my_lisp::Session` (!Send) on a dedicated worker thread.
pub struct ManagedReplSession {
    sender: Mutex<std::sync::mpsc::Sender<ReplCommand>>,
}

impl Default for ManagedReplSession {
    fn default() -> Self {
        Self::new()
    }
}

impl ManagedReplSession {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<ReplCommand>();
        std::thread::Builder::new()
            .name("my-idea-repl".into())
            .spawn(move || {
                let mut session = ReplSession::default();
                // Installed before any plugin ever loads: init.lisp/plugins/*.lisp
                // (loaded below and again on explicit reload) can call
                // editor/register-command, editor/keymap, editor/on etc. from
                // the very first line they evaluate.
                let registry = EditorCommandRegistry::new();
                registry.install_into(&mut session);
                while let Ok(cmd) = rx.recv() {
                    match cmd {
                        ReplCommand::Evaluate { source, mode, reply } => {
                            let res = evaluate_source_in_session(&mut session, &source, mode.as_deref());
                            let _ = reply.send(res);
                        }
                        ReplCommand::LoadPlugins { config_dir, reply } => {
                            let report = crate::plugins::load_plugins(&mut session, &config_dir);
                            let _ = reply.send(report);
                        }
                        ReplCommand::DispatchKey { key, state, reply } => {
                            let res = registry.dispatch_key(&mut session, &key, &state);
                            let _ = reply.send(res);
                        }
                        ReplCommand::InvokeCommand { name, state, reply } => {
                            let res = registry.invoke_command(&mut session, &name, &state);
                            let _ = reply.send(res);
                        }
                        ReplCommand::EmitEvent { event, state, reply } => {
                            let res = registry.emit_event(&mut session, &event, &state);
                            let _ = reply.send(res);
                        }
                        ReplCommand::ListCommands { reply } => {
                            let _ = reply.send(registry.list_commands());
                        }
                        ReplCommand::ListKeymaps { reply } => {
                            let _ = reply.send(registry.list_keymaps());
                        }
                    }
                }
            })
            .expect("failed to spawn REPL actor thread");

        Self {
            sender: Mutex::new(tx),
        }
    }

    pub fn evaluate(&self, source: &str, mode: Option<&str>) -> Result<LispEvaluation, String> {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let cmd = ReplCommand::Evaluate {
            source: source.to_string(),
            mode: mode.map(str::to_string),
            reply: reply_tx,
        };
        self.sender
            .lock()
            .map_err(|_| "repl actor channel poisoned".to_string())?
            .send(cmd)
            .map_err(|e| format!("failed to send to repl actor: {e}"))?;

        reply_rx
            .recv()
            .map_err(|e| format!("failed to receive from repl actor: {e}"))?
    }

    /// Loads `init.lisp`/`plugins/*.lisp` from `config_dir` into the live
    /// session on the actor thread -- called once at startup, and again
    /// whenever an explicit reload is requested.
    ///
    /// Завантажує `init.lisp`/`plugins/*.lisp` з `config_dir` у живу сесію
    /// на потоці-акторі — викликається один раз при старті, і знову за
    /// явним запитом reload.
    pub fn load_plugins(&self, config_dir: &Path) -> crate::plugins::PluginLoadReport {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let cmd = ReplCommand::LoadPlugins {
            config_dir: config_dir.to_path_buf(),
            reply: reply_tx,
        };
        let sent = self
            .sender
            .lock()
            .map(|sender| sender.send(cmd).is_ok())
            .unwrap_or(false);
        if !sent {
            return crate::plugins::PluginLoadReport::default();
        }
        reply_rx.recv().unwrap_or_default()
    }

    /// Dispatches a key press to whatever command a Lisp plugin bound it to
    /// via `editor/keymap` — an unbound key or a keymap pointing at a
    /// command that no longer exists are both fail-closed errors (issue #11).
    pub fn dispatch_key(&self, key: &str, state: EditorState) -> Result<EditorEffect, String> {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let cmd = ReplCommand::DispatchKey { key: key.to_string(), state, reply: reply_tx };
        self.sender
            .lock()
            .map_err(|_| "repl actor channel poisoned".to_string())?
            .send(cmd)
            .map_err(|e| format!("failed to send to repl actor: {e}"))?;
        reply_rx
            .recv()
            .map_err(|e| format!("failed to receive from repl actor: {e}"))?
    }

    /// Invokes a command a Lisp plugin registered via `editor/register-command`
    /// by name — used both for command-palette-style invocation and as the
    /// target a keymap resolves to.
    pub fn invoke_command(&self, name: &str, state: EditorState) -> Result<EditorEffect, String> {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let cmd = ReplCommand::InvokeCommand { name: name.to_string(), state, reply: reply_tx };
        self.sender
            .lock()
            .map_err(|_| "repl actor channel poisoned".to_string())?
            .send(cmd)
            .map_err(|e| format!("failed to send to repl actor: {e}"))?;
        reply_rx
            .recv()
            .map_err(|e| format!("failed to receive from repl actor: {e}"))?
    }

    /// Runs every handler a Lisp plugin subscribed to `event` via
    /// `editor/on`, isolated — one handler failing never stops the rest.
    pub fn emit_event(&self, event: &str, state: EditorState) -> Vec<Result<EditorEffect, String>> {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let cmd = ReplCommand::EmitEvent { event: event.to_string(), state, reply: reply_tx };
        let sent = self
            .sender
            .lock()
            .map(|sender| sender.send(cmd).is_ok())
            .unwrap_or(false);
        if !sent {
            return Vec::new();
        }
        reply_rx.recv().unwrap_or_default()
    }

    /// Lists every command name currently registered by a loaded plugin.
    pub fn list_commands(&self) -> Vec<String> {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let cmd = ReplCommand::ListCommands { reply: reply_tx };
        let sent = self
            .sender
            .lock()
            .map(|sender| sender.send(cmd).is_ok())
            .unwrap_or(false);
        if !sent {
            return Vec::new();
        }
        reply_rx.recv().unwrap_or_default()
    }

    /// Lists every `(key, command-name)` keymap binding currently registered
    /// by a loaded plugin.
    pub fn list_keymaps(&self) -> Vec<(String, String)> {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let cmd = ReplCommand::ListKeymaps { reply: reply_tx };
        let sent = self
            .sender
            .lock()
            .map(|sender| sender.send(cmd).is_ok())
            .unwrap_or(false);
        if !sent {
            return Vec::new();
        }
        reply_rx.recv().unwrap_or_default()
    }
}

/// Helper to evaluate code in a given `ReplSession` with an optional language mode string.
pub fn evaluate_source_in_session(
    repl: &mut ReplSession,
    source: &str,
    mode: Option<&str>,
) -> Result<LispEvaluation, String> {
    let mode_str = mode.unwrap_or("my-lisp");
    let source_mode = if mode_str == "markdown" {
        SourceMode::Literate
    } else {
        SourceMode::PureLisp
    };
    repl.evaluate_mode(source, source_mode)
}
