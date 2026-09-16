//! Production-level integration witness for #41 (IDE-EMACS-BRIDGE-1).
//!
//! `editor_api_contract.rs`/`editor_hooks_contract.rs` already proved
//! `EditorCommandRegistry` works against a bare `ReplSession` — but that
//! was never the actual gap. The gap was that production's
//! `ManagedReplSession` (the actor-thread-backed session every real Tauri
//! command in `lib.rs` actually talks to) never installed the registry
//! into its own session. This test goes through `ManagedReplSession`
//! itself — the same object `editor_dispatch_key`/`editor_invoke_command`/
//! `editor_emit_event` in `lib.rs` are thin wrappers around — so a
//! regression here (e.g. someone reintroducing a second, uninstalled
//! session) fails exactly where the real bug used to be.
//!
//! Uses the issue's own demonstration plugin verbatim.

use my_idea_lib::ManagedReplSession;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestConfigDir(PathBuf);

impl TestConfigDir {
    fn with_plugin(source: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("my-idea-emacs-bridge-witness-{nonce}"));
        let plugins_dir = path.join("plugins");
        fs::create_dir_all(&plugins_dir).expect("config dir should be created");
        fs::write(plugins_dir.join("bracket.lisp"), source).expect("plugin file should be written");
        Self(path)
    }
}

impl Drop for TestConfigDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const BRACKET_PLUGIN: &str = r#"
(editor/register-command "bracket-selection"
  (lambda ()
    (editor/replace-selection
      (string-append "["
        (string-append (editor/selection) "]"))))
  "Обгорнути виділення")

(editor/keymap "Ctrl-[" "bracket-selection")
"#;

#[test]
fn plugin_lisp_registers_a_command_in_the_actual_production_session_and_it_changes_editor_state() {
    let config = TestConfigDir::with_plugin(BRACKET_PLUGIN);
    let session = ManagedReplSession::new();

    let report = session.load_plugins(&config.0);
    assert!(report.failures.is_empty(), "plugin should load cleanly: {:?}", report.failures);
    assert_eq!(report.loaded.len(), 1);

    assert_eq!(session.list_commands(), vec!["bracket-selection".to_string()]);
    assert_eq!(
        session.list_keymaps(),
        vec![("Ctrl-[".to_string(), "bracket-selection".to_string())]
    );

    let state = my_idea_lib::editor_api::EditorState {
        buffer: "hello world of lisp".to_string(),
        selection: "world".to_string(),
    };

    let effect = session
        .invoke_command("bracket-selection", state.clone())
        .expect("registered command should run through the production session");
    assert_eq!(effect.replacement, Some("[world]".to_string()));

    let via_key = session
        .dispatch_key("Ctrl-[", state)
        .expect("the keymap binding should dispatch through the production session");
    assert_eq!(via_key.replacement, Some("[world]".to_string()));
}

#[test]
fn unknown_command_key_and_event_all_fail_closed_on_the_production_session() {
    let config = TestConfigDir::with_plugin(BRACKET_PLUGIN);
    let session = ManagedReplSession::new();
    session.load_plugins(&config.0);

    assert!(session.invoke_command("does-not-exist", Default::default()).is_err());
    assert!(session.dispatch_key("Ctrl-Never-Bound", Default::default()).is_err());
    // An unsubscribed event is not an error -- zero handlers ran, isolated
    // per handler the same way plugin loading isolates per file.
    assert!(session.emit_event("no-such-event", Default::default()).is_empty());
}

#[test]
fn reloading_the_same_plugin_does_not_duplicate_its_command_or_keymap() {
    let config = TestConfigDir::with_plugin(BRACKET_PLUGIN);
    let session = ManagedReplSession::new();

    session.load_plugins(&config.0);
    session.load_plugins(&config.0);

    assert_eq!(session.list_commands(), vec!["bracket-selection".to_string()]);
    assert_eq!(session.list_keymaps().len(), 1);
}
