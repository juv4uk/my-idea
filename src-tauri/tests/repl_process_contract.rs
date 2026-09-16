//! RED contract for #31 (IDE-REPL-SURFACE-1): the console must wrap the
//! *real* `my-lisp` CLI binary (built from the `external/my-lisp` git
//! submodule) as a live subprocess, not reimplement its REPL/surface logic
//! in my-idea. This proves both halves: a generic interactive-process
//! wrapper, and that the actual submodule binary behaves as the real REPL
//! (banner, `:мова` surface switching, localized presentation) when driven
//! through it.

use my_idea_lib::repl_process::{
    my_idea_repo_root, my_lisp_pinned_sha, resolve_my_lisp_binary, resolve_or_fetch_my_lisp_binary,
    ReplProcess, ReplProcessStream,
};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

#[test]
fn repl_process_forwards_lines_written_to_a_simple_echo_command() {
    let (tx, rx) = channel();
    let process = ReplProcess::spawn(Path::new("cat"), &[], move |line| {
        let _ = tx.send(line);
    })
    .expect("spawning `cat` as a stand-in interactive process should succeed");

    process.write_line("hello from the test").expect("write_line should succeed");

    let received = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("the echoed line should arrive on stdout");
    assert_eq!(received.stream, ReplProcessStream::Stdout);
    assert_eq!(received.line, "hello from the test");
}

#[test]
fn resolve_my_lisp_binary_fails_closed_when_submodule_is_not_checked_out() {
    let bogus_root = std::env::temp_dir().join("my-idea-repl-process-contract-nonexistent-root");
    let result = resolve_my_lisp_binary(&bogus_root);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("submodule"));
}

#[test]
fn resolve_my_lisp_binary_builds_the_real_submodule_repl_and_it_behaves_as_documented() {
    let binary = resolve_my_lisp_binary(&my_idea_repo_root())
        .expect("building my-lisp from the external/my-lisp submodule should succeed");
    assert!(binary.exists());

    let (tx, rx) = channel();
    let process = ReplProcess::spawn(&binary, &[], move |line| {
        let _ = tx.send(line);
    })
    .expect("spawning the real my-lisp REPL binary should succeed");

    // The real banner (docs/repl-surfaces.md in external/my-lisp) must show
    // up verbatim — nothing in my-idea generates this text.
    let banner = rx
        .recv_timeout(Duration::from_secs(10))
        .expect("the real REPL should print its startup banner");
    assert!(banner.line.contains("my-lisp REPL"));

    process.write_line(":мова ук").expect("write_line should succeed");
    process.write_line("(атом? (quote мама))").expect("write_line should succeed");

    let mut lines = Vec::new();
    while lines.len() < 4 {
        match rx.recv_timeout(Duration::from_secs(10)) {
            Ok(line) => lines.push(line.line),
            Err(_) => break,
        }
    }
    let joined = lines.join("\n");
    assert!(joined.contains("українська"), "surface switch reply missing, got: {joined:?}");
    assert!(joined.contains("істина"), "localized ukrainian output missing, got: {joined:?}");
}

#[test]
fn my_lisp_pinned_sha_is_recorded_at_build_time_from_the_checked_out_submodule() {
    // src-tauri/build.rs reads `git -C external/my-lisp rev-parse HEAD` at
    // compile time; this dev/CI build always has the submodule checked out
    // (my-idea's own Cargo dependency requires it), so the pin must be a
    // real 40-hex-character commit, never empty.
    let sha = my_lisp_pinned_sha();
    assert_eq!(sha.len(), 40, "expected a full commit SHA, got: {sha:?}");
    assert!(sha.chars().all(|c| c.is_ascii_hexdigit()), "not a hex SHA: {sha:?}");
}

#[test]
fn resolve_or_fetch_uses_the_local_submodule_when_present_no_network_needed() {
    let cache_dir = std::env::temp_dir().join("my-idea-repl-process-contract-unused-cache");
    let binary = resolve_or_fetch_my_lisp_binary(&my_idea_repo_root(), &cache_dir)
        .expect("a local external/my-lisp checkout should be used directly");
    assert!(binary.exists());
    // The fallback cache must not have been touched — the local submodule
    // path was available, so no clone should ever have been attempted.
    assert!(!cache_dir.join("my-lisp").exists());
}

#[test]
fn resolve_or_fetch_clones_the_pinned_commit_directly_from_github_when_no_local_submodule() {
    // Simulates a packaged install: repo_root has no external/my-lisp at
    // all (a real installed app's directory never does), so the only
    // source of truth left is the pinned SHA baked in at compile time plus
    // my-lisp's actual GitHub URL — a git reference, not a machine-local path.
    let bogus_root = std::env::temp_dir().join("my-idea-repl-process-contract-no-submodule-here");
    let cache_dir = std::env::temp_dir().join(format!(
        "my-idea-repl-process-contract-fetch-cache-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&cache_dir);

    let binary = resolve_or_fetch_my_lisp_binary(&bogus_root, &cache_dir)
        .expect("cloning the pinned commit from github.com/juv4uk/my-lisp should succeed");
    assert!(binary.exists());

    let (tx, rx) = channel();
    let _process = ReplProcess::spawn(&binary, &[], move |line| {
        let _ = tx.send(line);
    })
    .expect("spawning the freshly cloned+built my-lisp REPL binary should succeed");
    let banner = rx
        .recv_timeout(Duration::from_secs(10))
        .expect("the real REPL should print its startup banner");
    assert!(banner.line.contains("my-lisp REPL"));

    let _ = std::fs::remove_dir_all(&cache_dir);
}
