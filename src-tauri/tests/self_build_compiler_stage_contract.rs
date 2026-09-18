//! RED contract for #13 compiler-stage evidence.
//!
//! Freshness is content/provenance based: the exact source and artifact hashes
//! plus the plan/source/compiler revisions must still match when Tauri consumes
//! the stage.

use my_idea_lib::self_build::{
    build_self_build_plan, verify_compiler_stage, CompilerStageEvidence, SelfBuildInputs,
};
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
        let root = std::env::temp_dir().join(format!("my-idea-self-build-evidence-{nonce}"));
        fs::create_dir_all(root.join("self-build")).unwrap();
        fs::create_dir_all(root.join("target/self-build")).unwrap();
        fs::write(root.join("self-build/my-idea.lisp"), b"(- 40 40)\n").unwrap();

        run(&root, &["init", "-q"]);
        run(&root, &["config", "user.email", "self-build@test.invalid"]);
        run(&root, &["config", "user.name", "self-build-test"]);
        run(&root, &["add", "self-build/my-idea.lisp"]);
        run(&root, &["commit", "-q", "-m", "fixture"]);
        Self(root)
    }

    fn head(&self) -> String {
        output(&self.0, &["rev-parse", "HEAD"])
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} must succeed");
}

fn output(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fixture() -> (TestRepo, my_idea_lib::self_build::SelfBuildPlan, CompilerStageEvidence) {
    let repo = TestRepo::new();
    let revision = repo.head();
    let compiler_revision = "3333333333333333333333333333333333333333";
    let plan = build_self_build_plan(SelfBuildInputs::new(
        &revision,
        true,
        "2222222222222222222222222222222222222222",
        "cml-compile",
        compiler_revision,
        "x86_64-linux",
        "rustc test",
        "cargo test",
        "bun-test",
        "tauri-test",
    ))
    .unwrap();

    let source_path = repo.0.join("self-build/my-idea.lisp");
    let artifact_path = repo.0.join("target/self-build/my-idea-x86_64-linux");
    fs::write(&artifact_path, b"fresh compiler artifact").unwrap();

    let evidence = CompilerStageEvidence::new(
        plan.digest(),
        &revision,
        "self-build/my-idea.lisp",
        hash(&fs::read(&source_path).unwrap()),
        "cml-compile",
        compiler_revision,
        "x86_64-linux",
        "target/self-build/my-idea-x86_64-linux",
        hash(&fs::read(&artifact_path).unwrap()),
    )
    .unwrap();

    (repo, plan, evidence)
}

#[test]
fn matching_compiler_stage_evidence_is_accepted() {
    let (repo, plan, evidence) = fixture();
    verify_compiler_stage(&repo.0, &plan, &evidence).unwrap();
}

#[test]
fn changed_artifact_is_rejected_as_stale() {
    let (repo, plan, evidence) = fixture();
    fs::write(
        repo.0.join("target/self-build/my-idea-x86_64-linux"),
        b"stale/tampered artifact",
    )
    .unwrap();

    let error = verify_compiler_stage(&repo.0, &plan, &evidence).unwrap_err();
    assert!(error.to_string().contains("artifact SHA-256"));
}

#[test]
fn changed_source_is_rejected_as_stale() {
    let (repo, plan, evidence) = fixture();
    fs::write(repo.0.join("self-build/my-idea.lisp"), b"(- 41 40)\n").unwrap();

    let error = verify_compiler_stage(&repo.0, &plan, &evidence).unwrap_err();
    assert!(error.to_string().contains("source SHA-256"));
}

#[test]
fn missing_artifact_is_rejected() {
    let (repo, plan, evidence) = fixture();
    fs::remove_file(repo.0.join("target/self-build/my-idea-x86_64-linux")).unwrap();

    let error = verify_compiler_stage(&repo.0, &plan, &evidence).unwrap_err();
    assert!(error.to_string().contains("artifact"));
}
