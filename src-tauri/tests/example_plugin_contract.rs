//! The example plugin shipped in `docs/examples/plugins/hello.lisp` (the
//! doc every user following the my-lisp plugin system is pointed at) must
//! actually load and behave as documented — a stale example that no longer
//! parses would be a broken first-touch experience, and this test is what
//! keeps it honest as the language/Editor API evolve.

use my_idea_lib::editor_api::{EditorCommandRegistry, EditorState};
use my_idea_lib::ReplSession;

#[test]
fn shipped_example_plugin_loads_and_its_command_and_hook_work() {
    let source = include_str!("../../docs/examples/plugins/hello.lisp");
    let mut repl = ReplSession::default();
    let registry = EditorCommandRegistry::new();
    registry.install_into(&mut repl);
    repl.evaluate(source)
        .expect("the shipped example plugin should evaluate cleanly");

    assert!(registry.has_command("bracket-selection"));
    assert!(registry.has_keymap("Ctrl-Shift-b"));

    let state = EditorState {
        buffer: "x".to_string(),
        selection: "hi".to_string(),
    };
    let effect = registry
        .invoke_command(&mut repl, "bracket-selection", &state)
        .expect("the example command should run");
    assert_eq!(effect.replacement, Some("[hi]".to_string()));

    let results = registry.emit_event(&mut repl, "after-open", &EditorState::default());
    assert_eq!(results.len(), 1, "the example's after-open hook should be registered");
    let effect = results[0].as_ref().expect("the example hook should run without error");
    assert!(effect.message.is_some(), "the example hook should emit a message");
}
