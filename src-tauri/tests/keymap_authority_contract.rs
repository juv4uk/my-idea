//! Integration-level witness for #51 (IDE-KEYMAP-AUTHORITY-1): a real
//! plugin's `editor/keymap` binding, combined with the app's one built-in
//! binding, resolved through the real `ManagedReplSession` +
//! `keymap_authority::resolve` — not just the pure-function unit tests in
//! `keymap_authority.rs` itself.
//!
//! This is exactly the combination `lib.rs`'s `resolve_keymaps` Tauri
//! command performs (`session.list_keymaps()` -> `Binding`s, plus the
//! frontend-reported built-ins, into `keymap_authority::resolve`), against
//! the real actor-thread session, the same way
//! `emacs_bridge_production_witness.rs` proved #41 through
//! `ManagedReplSession` rather than a bare `EditorCommandRegistry`.

use my_idea_lib::keymap_authority::{resolve, Binding, Source};
use my_idea_lib::ManagedReplSession;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestConfigDir(PathBuf);

impl TestConfigDir {
    fn with_plugin(source: &str) -> Self {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("my-idea-keymap-authority-witness-{nonce}"));
        let plugins_dir = path.join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();
        fs::write(plugins_dir.join("plugin.lisp"), source).unwrap();
        Self(path)
    }
}

impl Drop for TestConfigDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn resolve_with_builtins(session: &ManagedReplSession) -> Vec<my_idea_lib::keymap_authority::Resolution> {
    let mut bindings: Vec<Binding> = session
        .list_keymaps()
        .into_iter()
        .map(|(key, command)| Binding { key, command, source: Source::UserPlugin })
        .collect();
    bindings.push(Binding {
        key: "Alt-.".to_string(),
        command: "go-to-definition".to_string(),
        source: Source::BuiltIn,
    });
    resolve(bindings)
}

#[test]
fn a_real_plugin_binding_that_does_not_collide_with_the_built_in_wins_cleanly() {
    let config = TestConfigDir::with_plugin(
        r#"(editor/register-command "greet" (lambda () (editor/message "hi")) "greet")
           (editor/keymap "Ctrl-g" "greet")"#,
    );
    let session = ManagedReplSession::new();
    session.load_plugins(&config.0);

    let resolutions = resolve_with_builtins(&session);
    let greet = resolutions.iter().find(|r| r.key == "Ctrl-g").expect("Ctrl-g should be resolved");
    assert_eq!(greet.winner.as_ref().unwrap().command, "greet");
    assert_eq!(greet.reason, "no collision");

    let go_to_def = resolutions.iter().find(|r| r.key == "Alt-.").expect("Alt-. should be resolved");
    assert_eq!(go_to_def.winner.as_ref().unwrap().command, "go-to-definition");
}

#[test]
fn a_real_plugin_that_tries_to_claim_f12_is_reported_host_reserved_not_silently_bound() {
    let config = TestConfigDir::with_plugin(
        r#"(editor/register-command "sneaky" (lambda () (editor/message "should never fire")) "sneaky")
           (editor/keymap "F12" "sneaky")"#,
    );
    let session = ManagedReplSession::new();
    session.load_plugins(&config.0);

    let resolutions = resolve_with_builtins(&session);
    let f12 = resolutions.iter().find(|r| r.key == "F12").expect("F12 should be resolved");
    assert!(f12.winner.is_none(), "a plugin must never win a host-reserved key");
    assert!(f12.reason.contains("host-reserved"));
    assert!(f12.candidates.iter().any(|c| c.command == "sneaky"), "the attempt must still be visible, not dropped");
}

#[test]
fn a_real_plugin_that_reuses_the_built_in_go_to_definition_key_overrides_it() {
    // Emacs' own precedence: a user's explicit keybinding wins over a
    // built-in bound to the same key ((global-set-key ...) in init.el
    // shadows Emacs' own defaults too) -- the collision is still fully
    // visible in `candidates`, just not silently invisible.
    let config = TestConfigDir::with_plugin(
        r#"(editor/register-command "custom" (lambda () (editor/message "custom")) "custom")
           (editor/keymap "Alt-." "custom")"#,
    );
    let session = ManagedReplSession::new();
    session.load_plugins(&config.0);

    let resolutions = resolve_with_builtins(&session);
    let alt_dot = resolutions.iter().find(|r| r.key == "Alt-.").unwrap();
    assert_eq!(
        alt_dot.winner.as_ref().unwrap().command,
        "custom",
        "a user plugin binding outranks the built-in bound to the same key"
    );
    assert_eq!(alt_dot.candidates.len(), 2);
}
