//! Integration tests for the `my-lisp` CLI binary.
//! Інтеграційні тести для CLI-бінарника `my-lisp`.
//! Integrationstests für die `my-lisp`-CLI-Binärdatei.
//!
//! These exercise the compiled binary as a black box (argv in, stdout/stderr/exit
//! code out) instead of calling internal functions directly, since main.rs itself
//! has no unit-testable functions — the behavior lives in argument handling and I/O.
//! Вони перевіряють скомпільований бінарник як чорну скриньку (argv на вході,
//! stdout/stderr/код виходу на виході), а не викликають внутрішні функції напряму,
//! бо main.rs не має власних функцій для unit-тестів — поведінка живе в обробці
//! аргументів та I/O.
//! Sie prüfen die kompilierte Binärdatei als Black Box (argv als Eingabe,
//! stdout/stderr/Exit-Code als Ausgabe) statt interne Funktionen direkt aufzurufen,
//! da main.rs selbst keine unit-testbaren Funktionen besitzt — das Verhalten steckt
//! in der Argumentverarbeitung und E/A.

use std::process::Command;

fn my_lisp() -> Command {
    Command::new(env!("CARGO_BIN_EXE_my-lisp"))
}

#[test]
fn version_flag_prints_the_crate_version() {
    let output = my_lisp().arg("--version").output().expect("binary should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), format!("my-lisp {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn short_version_flags_match_the_long_form() {
    for flag in ["-V", "-v"] {
        let output = my_lisp().arg(flag).output().expect("binary should run");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), format!("my-lisp {}", env!("CARGO_PKG_VERSION")));
    }
}

#[test]
fn help_flag_prints_usage() {
    let output = my_lisp().arg("--help").output().expect("binary should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: my-lisp [file]"));
}

#[test]
fn running_a_source_file_prints_its_result() {
    let dir = std::env::temp_dir();
    let path = dir.join("my-lisp-cli-test-ok.my");
    std::fs::write(&path, "(+ 1 2)").expect("should write temp file");

    let output = my_lisp().arg(&path).output().expect("binary should run");
    let _ = std::fs::remove_file(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "3");
}

#[test]
fn running_a_file_with_an_evaluation_error_exits_nonzero() {
    let dir = std::env::temp_dir();
    let path = dir.join("my-lisp-cli-test-eval-error.my");
    std::fs::write(&path, "(car '())").expect("should write temp file");

    let output = my_lisp().arg(&path).output().expect("binary should run");
    let _ = std::fs::remove_file(&path);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("Error:"));
}

#[test]
fn running_a_file_with_a_parse_error_exits_nonzero() {
    let dir = std::env::temp_dir();
    let path = dir.join("my-lisp-cli-test-parse-error.my");
    std::fs::write(&path, "(1 2").expect("should write temp file");

    let output = my_lisp().arg(&path).output().expect("binary should run");
    let _ = std::fs::remove_file(&path);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("Parse error:"));
}

#[test]
fn running_a_missing_file_reports_a_read_error() {
    let output = my_lisp()
        .arg("this-file-does-not-exist-my-lisp-cli.my")
        .output()
        .expect("binary should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("Error reading file"));
}

#[test]
fn core_lib_is_preloaded_before_running_a_file() {
    // lib/core.my defines `identity`; if the CLI stopped injecting core.my this
    // would fail with an "unknown symbol" evaluation error instead of returning 5.
    // lib/core.my визначає `identity`; якби CLI перестав вставляти core.my, це б
    // провалилось помилкою "unknown symbol" замість повернення 5.
    // lib/core.my definiert `identity`; würde die CLI core.my nicht mehr einspeisen,
    // schlüge dies mit einem "unknown symbol"-Fehler fehl statt 5 zurückzugeben.
    let dir = std::env::temp_dir();
    let path = dir.join("my-lisp-cli-test-core-lib.my");
    std::fs::write(&path, "(identity 5)").expect("should write temp file");

    let output = my_lisp().arg(&path).output().expect("binary should run");
    let _ = std::fs::remove_file(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "5");
}
