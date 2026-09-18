//! RED contract for #13: the Cargo/Tauri self-build gate must independently
//! re-verify the compiler-stage source/artifact hashes and source revision.

#[path = "../build_support.rs"]
mod build_support;

use build_support::{verify_compiler_stage_gate, CompilerStageGate};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct TestRepo(PathBuf);

impl TestRepo {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("my-idea-tauri-gate-{nonce}"));
        fs::create_dir_all(root.join("self-build")).unwrap();
        fs::create_dir_all(root.join("target/self-build")).unwrap();
        fs::write(root.join(".gitignore"), b"/target/\n").unwrap();
        fs::write(root.join("self-build/my-idea.lisp"), b"(- 40 40)\n").unwrap();

        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "gate@test.invalid"]);
        git(&root, &["config", "user.name", "gate-test"]);
        git(&root, &["add", ".gitignore", "self-build/my-idea.lisp"]);
        git(&root, &["commit", "-q", "-m", "fixture"]);
        Self(root)
    }

    fn head(&self) -> String {
        git_output(&self.0, &["rev-parse", "HEAD"])
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn git(root: &Path, args: &[&str]) {
    assert!(Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()
        .unwrap()
        .success());
}

fn git_output(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn hash(path: &Path) -> String {
    format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
}

fn fixture() -> (TestRepo, CompilerStageGate) {
    let repo = TestRepo::new();
    let source = repo.0.join("self-build/my-idea.lisp");
    let artifact = repo.0.join("target/self-build/my-idea-x86_64-linux");
    fs::write(&artifact, b"compiler artifact").unwrap();

    let gate = CompilerStageGate::new(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        repo.head(),
        "self-build/my-idea.lisp",
        hash(&source),
        "3333333333333333333333333333333333333333",
        "x86_64-linux",
        "target/self-build/my-idea-x86_64-linux",
        hash(&artifact),
    )
    .unwrap();

    (repo, gate)
}

#[test]
fn matching_gate_is_accepted() {
    let (repo, gate) = fixture();
    verify_compiler_stage_gate(&repo.0, &gate).unwrap();
}

#[test]
fn missing_artifact_is_rejected_before_tauri_build() {
    let (repo, gate) = fixture();
    fs::remove_file(repo.0.join("target/self-build/my-idea-x86_64-linux")).unwrap();
    assert!(verify_compiler_stage_gate(&repo.0, &gate)
        .unwrap_err()
        .contains("artifact"));
}

#[test]
fn changed_artifact_is_rejected_before_tauri_build() {
    let (repo, gate) = fixture();
    fs::write(
        repo.0.join("target/self-build/my-idea-x86_64-linux"),
        b"different artifact",
    )
    .unwrap();
    assert!(verify_compiler_stage_gate(&repo.0, &gate)
        .unwrap_err()
        .contains("artifact SHA-256"));
}

#[test]
fn changed_source_is_rejected_before_tauri_build() {
    let (repo, gate) = fixture();
    fs::write(repo.0.join("self-build/my-idea.lisp"), b"(- 41 40)\n").unwrap();
    assert!(verify_compiler_stage_gate(&repo.0, &gate)
        .unwrap_err()
        .contains("source SHA-256"));
}
