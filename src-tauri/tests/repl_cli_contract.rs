//! RED contract for #7: the desktop IDE needs one persistent REPL session
//! and a console-launch contract.  This intentionally names the public API
//! before production code exists.

use my_idea_lib::{parse_startup_target, ReplSession, StartupTarget};
use std::path::PathBuf;

#[test]
fn console_launch_without_path_opens_default_workspace() {
    assert_eq!(parse_startup_target(Vec::<String>::new()).unwrap(), StartupTarget::DefaultWorkspace);
}

#[test]
fn console_launch_accepts_one_startup_path() {
    let target = parse_startup_target(["examples/hello.my".to_owned()]).unwrap();
    assert_eq!(target, StartupTarget::Path(PathBuf::from("examples/hello.my")));
}

#[test]
fn console_launch_rejects_ambiguous_extra_paths() {
    let error = parse_startup_target(["one.my".to_owned(), "two.my".to_owned()]).unwrap_err();
    assert!(error.contains("at most one"));
}

#[test]
fn repl_keeps_one_environment_across_evaluations() {
    let mut repl = ReplSession::default();
    repl.evaluate("(define repl-x 41)").unwrap();
    let result = repl.evaluate("(+ repl-x 1)").unwrap();
    assert_eq!(result.value, "42");
}
