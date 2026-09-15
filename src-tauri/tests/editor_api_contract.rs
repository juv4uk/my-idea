//! RED contract for #9 (IDE-LISP-API-1): minimal Editor API exposed to my-lisp.
//!
//! A plugin written in canonical .lisp registers and invokes an editor command
//! through explicit native mechanisms without UI-owned Lisp semantics.

use my_idea_lib::editor_api::{EditorCommandRegistry, EditorEffect, EditorState};
use my_idea_lib::ReplSession;

#[test]
fn plugin_registers_command_and_modifies_selection_via_editor_api() {
    let mut repl = ReplSession::default();
    let registry = EditorCommandRegistry::new();
    registry.install_into(&mut repl);

    // Canonical Lisp plugin registering a command
    let plugin_source = r#"
        (editor/register-command "bracket-selection"
          (lambda ()
            (editor/replace-selection (string-append "[" (string-append (editor/selection) "]"))))
          "Wrap active selection in brackets")
    "#;

    repl.evaluate(plugin_source).expect("plugin evaluation should succeed");

    assert!(registry.has_command("bracket-selection"));
    assert_eq!(registry.list_commands(), vec!["bracket-selection".to_string()]);

    let state = EditorState {
        buffer: "hello world of lisp".to_string(),
        selection: "world".to_string(),
    };

    let effect = registry
        .invoke_command(&mut repl, "bracket-selection", &state)
        .expect("command invocation should succeed");

    assert_eq!(effect.replacement, Some("[world]".to_string()));
    assert_eq!(effect.message, None);
}

#[test]
fn plugin_can_read_buffer_and_emit_editor_messages() {
    let mut repl = ReplSession::default();
    let registry = EditorCommandRegistry::new();
    registry.install_into(&mut repl);

    let plugin_source = r#"
        (editor/register-command "greet-buffer"
          (lambda ()
            (editor/message (string-append "Buffer: " (editor/buffer-text))))
          "Emit greeting with buffer text")
    "#;

    repl.evaluate(plugin_source).expect("plugin evaluation should succeed");

    let state = EditorState {
        buffer: "my-code.my".to_string(),
        selection: String::new(),
    };

    let effect = registry
        .invoke_command(&mut repl, "greet-buffer", &state)
        .expect("command invocation should succeed");

    assert_eq!(effect.message, Some("Buffer: my-code.my".to_string()));
    assert_eq!(effect.replacement, None);
}

#[test]
fn unknown_command_invocation_fails_closed() {
    let mut repl = ReplSession::default();
    let registry = EditorCommandRegistry::new();
    registry.install_into(&mut repl);

    let state = EditorState {
        buffer: String::new(),
        selection: String::new(),
    };

    let result = registry.invoke_command(&mut repl, "non-existent-command", &state);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unknown command"));
}

#[test]
fn editor_api_cannot_access_unauthorized_host_capabilities() {
    let mut repl = ReplSession::default();
    let registry = EditorCommandRegistry::new();
    registry.install_into(&mut repl);

    // The capability-free Lisp core must not have raw filesystem / shell access
    let bad_source = "(read-file \"/etc/passwd\")";
    let result = repl.evaluate(bad_source);
    assert!(result.is_err());
}
