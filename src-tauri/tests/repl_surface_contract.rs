//! RED contract for #31 (IDE-REPL-SURFACE-1): terminal-style REPL console
//! must be able to switch my-lisp's own human language surface (uk/en/sa/core)
//! without losing user-defined bindings. The uk/sa surface content is pulled
//! directly from the `external/my-lisp` submodule, not reimplemented here.

use my_idea_lib::repl_surface::ReplSurface;
use my_idea_lib::{ManagedReplSession, ReplSession};

#[test]
fn default_surface_is_core() {
    let repl = ReplSession::default();
    assert_eq!(repl.surface(), ReplSurface::Core);
}

#[test]
fn core_surface_presents_canonical_output_not_localized() {
    // Stable surface peers (like `атом?`) are already bound at core-library
    // bootstrap regardless of surface (ADR-005: "core is not English") — the
    // surface layer governs *output presentation* and candidate/missing name
    // promotion, not whether a stable peer name resolves at all.
    let mut repl = ReplSession::default();
    let result = repl
        .evaluate("(атом? 'мама)")
        .expect("stable surface peers are bound regardless of surface");
    assert_eq!(result.value, "t");
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
fn managed_session_defaults_to_core_and_can_switch_surface_across_actor_thread() {
    let managed = ManagedReplSession::default();
    assert_eq!(managed.surface_status().0, "core");

    let (code, _title) = managed
        .switch_surface("ук")
        .expect("actor thread should switch surface");
    assert_eq!(code, "ук");
    assert_eq!(managed.surface_status().0, "ук");

    let result = managed
        .evaluate("(атом? 'мама)", None)
        .expect("ukrainian surface name should evaluate on the actor thread");
    assert_eq!(result.value, "істина");
}

#[test]
fn managed_session_switch_surface_rejects_unknown_code() {
    let managed = ManagedReplSession::default();
    let result = managed.switch_surface("xx");
    assert!(result.is_err());
}

#[test]
fn surface_parse_accepts_documented_codes() {
    assert_eq!(ReplSurface::parse("core"), Some(ReplSurface::Core));
    assert_eq!(ReplSurface::parse("en"), Some(ReplSurface::English));
    assert_eq!(ReplSurface::parse("uk"), Some(ReplSurface::Ukrainian));
    assert_eq!(ReplSurface::parse("ук"), Some(ReplSurface::Ukrainian));
    assert_eq!(ReplSurface::parse("sa"), Some(ReplSurface::Sanskrit));
}
