use std::path::PathBuf;
use std::sync::Mutex;
use my_lisp_literate::SourceMode;
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
                while let Ok(cmd) = rx.recv() {
                    match cmd {
                        ReplCommand::Evaluate { source, mode, reply } => {
                            let res = evaluate_source_in_session(&mut session, &source, mode.as_deref());
                            let _ = reply.send(res);
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


