//! RED contract for #11 (IDE-LISP-HOOKS-1): let my-lisp own editor hooks
//! and keymaps -- `editor/keymap` binds a key to a command name,
//! `editor/on` subscribes a handler to an editor event. Dispatch/emit are
//! host mechanism; behavior stays Lisp-owned.

use my_idea_lib::editor_api::{EditorCommandRegistry, EditorState};
use my_idea_lib::ReplSession;

#[test]
fn plugin_binds_key_and_command_runs_via_dispatch() {
    let mut repl = ReplSession::default();
    let registry = EditorCommandRegistry::new();
    registry.install_into(&mut repl);

    repl.evaluate(
        r#"
        (editor/register-command "greet"
          (lambda () (editor/message "hello from keymap"))
          "Greets via keymap")
        (editor/keymap "Ctrl-g" "greet")
        "#,
    )
    .expect("plugin evaluation should succeed");

    let state = EditorState::default();
    let effect = registry
        .dispatch_key(&mut repl, "Ctrl-g", &state)
        .expect("bound key should dispatch successfully");

    assert_eq!(effect.message, Some("hello from keymap".to_string()));
}

#[test]
fn plugin_subscribes_to_event_and_handler_runs_via_emit() {
    let mut repl = ReplSession::default();
    let registry = EditorCommandRegistry::new();
    registry.install_into(&mut repl);

    repl.evaluate(
        r#"
        (editor/on "selection-changed" "log-selection"
          (lambda () (editor/message (string-append "selected: " (editor/selection)))))
        "#,
    )
    .expect("plugin evaluation should succeed");

    let state = EditorState {
        buffer: "hello world".to_string(),
        selection: "world".to_string(),
    };
    let results = registry.emit_event(&mut repl, "selection-changed", &state);

    assert_eq!(results.len(), 1);
    let effect = results[0].as_ref().expect("handler should not fail");
    assert_eq!(effect.message, Some("selected: world".to_string()));
}

#[test]
fn dispatching_an_unbound_key_fails_closed() {
    let mut repl = ReplSession::default();
    let registry = EditorCommandRegistry::new();
    registry.install_into(&mut repl);

    let state = EditorState::default();
    let result = registry.dispatch_key(&mut repl, "Ctrl-x", &state);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("no command bound"));
}

#[test]
fn keymap_bound_to_an_unregistered_command_fails_closed_at_dispatch() {
    let mut repl = ReplSession::default();
    let registry = EditorCommandRegistry::new();
    registry.install_into(&mut repl);

    repl.evaluate(r#"(editor/keymap "Ctrl-z" "never-registered")"#)
        .expect("keymap binding should succeed even before the command exists");

    let state = EditorState::default();
    let result = registry.dispatch_key(&mut repl, "Ctrl-z", &state);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unknown command"));
}

#[test]
fn reload_does_not_duplicate_keymap_or_event_handler() {
    let mut repl = ReplSession::default();
    let registry = EditorCommandRegistry::new();
    registry.install_into(&mut repl);

    let plugin_source = r#"
        (editor/register-command "greet"
          (lambda () (editor/message "hi"))
          "Greets")
        (editor/keymap "Ctrl-g" "greet")
        (editor/on "selection-changed" "log-selection"
          (lambda () (editor/message "seen")))
        "#;

    // Simulates an explicit plugin reload: the same source evaluated twice.
    repl.evaluate(plugin_source).expect("first load should succeed");
    repl.evaluate(plugin_source).expect("reload should succeed");

    assert_eq!(registry.list_keymaps(), vec![("Ctrl-g".to_string(), "greet".to_string())]);
    assert_eq!(registry.handler_count("selection-changed"), 1);

    let state = EditorState::default();
    let results = registry.emit_event(&mut repl, "selection-changed", &state);
    assert_eq!(results.len(), 1, "handler must fire exactly once, not once per reload");
}
