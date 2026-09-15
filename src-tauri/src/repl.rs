use std::path::PathBuf;
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
        let (result, forms) = my_lisp_literate::eval_literate(source, SourceMode::PureLisp, &mut self.session)
            .map_err(|error| error.to_string())?;

        Ok(LispEvaluation {
            value: result.value.to_string(),
            output: result.output,
            ast: format!("{forms:#?}"),
            engine: "my-lisp · Rust",
        })
    }
}
