//! RED contract for #7: the desktop IDE needs one persistent REPL session
//! and a console-launch contract.  This intentionally names the public API
//! before production code exists.

use my_idea_lib::{
    parse_startup_target, resolve_initial_workspace,
    ManagedReplSession, ReplSession, StartupTarget,
};
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

#[test]
fn managed_repl_session_evaluates_and_persists_state() {
    let managed = ManagedReplSession::default();
    managed.evaluate("(define x 99)", None).unwrap();
    let res = managed.evaluate("(+ x 1)", None).unwrap();
    assert_eq!(res.value, "100");
}


#[test]
fn resolve_initial_workspace_from_startup_target() {
    let target = StartupTarget::Path(PathBuf::from("src"));
    let resolved = resolve_initial_workspace(target);
    assert!(resolved.is_some());
    assert!(resolved.unwrap().ends_with("src"));

    assert_eq!(resolve_initial_workspace(StartupTarget::DefaultWorkspace), None);
}

