//! RED contract for #10 (IDE-LISP-PLUGINS-1): load `init.lisp` and
//! `plugins/*.lisp` into the persistent my-lisp session with deterministic
//! order, isolated per-plugin failure, and explicit reload.

use my_idea_lib::plugins::load_plugins;
use my_idea_lib::ReplSession;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestConfigDir(PathBuf);

impl TestConfigDir {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("my-idea-plugins-{label}-{nonce}"));
        fs::create_dir_all(path.join("plugins")).expect("config dir should be created");
        Self(path)
    }

    fn write_plugin(&self, name: &str, source: &str) {
        fs::write(self.0.join("plugins").join(name), source).expect("plugin file should be written");
    }

    fn write_init(&self, source: &str) {
        fs::write(self.0.join("init.lisp"), source).expect("init.lisp should be written");
    }
}

impl Drop for TestConfigDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn plugins_load_in_deterministic_filename_order() {
    let config = TestConfigDir::new("order");
    config.write_plugin("c-third.lisp", "(quote ok)");
    config.write_plugin("a-first.lisp", "(quote ok)");
    config.write_plugin("b-second.lisp", "(quote ok)");

    let mut repl = ReplSession::default();
    let report = load_plugins(&mut repl, &config.0);

    let names: Vec<String> = report
        .loaded
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        vec!["a-first.lisp".to_string(), "b-second.lisp".to_string(), "c-third.lisp".to_string()]
    );
    assert!(report.failures.is_empty());
}

#[test]
fn init_lisp_loads_before_any_plugin_and_its_definitions_are_visible_to_plugins() {
    let config = TestConfigDir::new("init-first");
    config.write_init("(define plugin-shared-base 10)");
    config.write_plugin(
        "uses-init.lisp",
        "(define plugin-shared-derived (+ plugin-shared-base 5))",
    );

    let mut repl = ReplSession::default();
    let report = load_plugins(&mut repl, &config.0);
    assert!(report.failures.is_empty());

    let result = repl
        .evaluate("plugin-shared-derived")
        .expect("derived value should be visible after load");
    assert_eq!(result.value, "15");
}

#[test]
fn one_broken_plugin_is_isolated_and_does_not_change_load_order_of_the_rest() {
    let config = TestConfigDir::new("isolated-failure");
    config.write_plugin("a-good.lisp", "(define plugin-a-loaded (quote t))");
    config.write_plugin("b-broken.lisp", "(this-is-not-a-bound-function 1 2 3)");
    config.write_plugin("c-good.lisp", "(define plugin-c-loaded (quote t))");

    let mut repl = ReplSession::default();
    let report = load_plugins(&mut repl, &config.0);

    let loaded_names: Vec<String> = report
        .loaded
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(loaded_names, vec!["a-good.lisp".to_string(), "c-good.lisp".to_string()]);

    assert_eq!(report.failures.len(), 1);
    assert_eq!(
        report.failures[0].path.file_name().unwrap().to_string_lossy(),
        "b-broken.lisp"
    );
    assert!(!report.failures[0].message.is_empty());

    // The session itself must survive a broken plugin -- no crash, still usable.
    assert_eq!(repl.evaluate("plugin-a-loaded").unwrap().value, "t");
    assert_eq!(repl.evaluate("plugin-c-loaded").unwrap().value, "t");
}

#[test]
fn only_dot_lisp_files_are_discovered_as_plugins() {
    let config = TestConfigDir::new("extension-filter");
    config.write_plugin("real.lisp", "(define plugin-real-loaded (quote t))");
    fs::write(config.0.join("plugins").join("notes.md"), "not lisp")
        .expect("non-lisp file should be written");

    let mut repl = ReplSession::default();
    let report = load_plugins(&mut repl, &config.0);

    assert_eq!(report.loaded.len(), 1);
    assert_eq!(
        report.loaded[0].file_name().unwrap().to_string_lossy(),
        "real.lisp"
    );
}

#[test]
fn reload_is_explicit_and_repeatable_without_crashing() {
    let config = TestConfigDir::new("reload");
    config.write_plugin("a.lisp", "(define plugin-reload-marker (quote loaded))");

    let mut repl = ReplSession::default();
    let first = load_plugins(&mut repl, &config.0);
    let second = load_plugins(&mut repl, &config.0);

    assert!(first.failures.is_empty());
    assert!(second.failures.is_empty());
    assert_eq!(first.loaded.len(), 1);
    assert_eq!(second.loaded.len(), 1);
}
