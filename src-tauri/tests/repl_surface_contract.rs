//! RED contract for #31 (IDE-REPL-SURFACE-1): terminal-style REPL console
//! must be able to switch my-lisp's own human language surface (uk/en/sa/core)
//! without losing user-defined bindings. The uk/sa surface content is pulled
//! directly from the `external/my-lisp` submodule, not reimplemented here.

use my_idea_lib::repl_surface::ReplSurface;
use my_idea_lib::ReplSession;

#[test]
fn default_surface_is_core() {
    let repl = ReplSession::default();
    assert_eq!(repl.surface(), ReplSurface::Core);
}

#[test]
fn core_surface_does_not_expose_ukrainian_names() {
    let mut repl = ReplSession::default();
    let result = repl.evaluate("(атом? 'мама)");
    assert!(result.is_err());
}

#[test]
fn switching_to_ukrainian_surface_exposes_ukrainian_names_and_localizes_output() {
    let mut repl = ReplSession::default();
    repl.switch_surface(ReplSurface::Ukrainian)
        .expect("switching to the ukrainian surface should succeed");
    assert_eq!(repl.surface(), ReplSurface::Ukrainian);

    let result = repl
        .evaluate("(атом? 'мама)")
        .expect("ukrainian surface name should evaluate");
    assert_eq!(result.value, "істина");
}

#[test]
fn switching_surface_preserves_user_definitions() {
    let mut repl = ReplSession::default();
    repl.evaluate("(define my-answer 42)")
        .expect("core define should succeed");

    repl.switch_surface(ReplSurface::Ukrainian)
        .expect("switching to the ukrainian surface should succeed");

    let result = repl
        .evaluate("my-answer")
        .expect("user definition should survive a surface switch");
    assert_eq!(result.value, "42");
}

#[test]
fn unknown_surface_is_rejected() {
    assert!(ReplSurface::parse("xx").is_none());
}

#[test]
fn surface_parse_accepts_documented_codes() {
    assert_eq!(ReplSurface::parse("core"), Some(ReplSurface::Core));
    assert_eq!(ReplSurface::parse("en"), Some(ReplSurface::English));
    assert_eq!(ReplSurface::parse("uk"), Some(ReplSurface::Ukrainian));
    assert_eq!(ReplSurface::parse("ук"), Some(ReplSurface::Ukrainian));
    assert_eq!(ReplSurface::parse("sa"), Some(ReplSurface::Sanskrit));
}
