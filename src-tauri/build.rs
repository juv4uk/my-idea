mod build_support;

use build_support::{verify_compiler_stage_gate, CompilerStageGate};
use std::env;
use std::path::Path;
use std::process::Command;

fn required_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("{name} is required by the self-build compiler-stage gate"))
}

fn verify_self_build_compiler_stage(repo_root: &Path) {
    if env::var("MY_IDEA_REQUIRE_COMPILER_STAGE").ok().as_deref() != Some("1") {
        return;
    }

    let gate = CompilerStageGate::new(
        required_env("MY_IDEA_SELF_BUILD_PLAN_DIGEST"),
        required_env("MY_IDEA_SELF_BUILD_SOURCE_REVISION"),
        required_env("MY_IDEA_SELF_BUILD_SOURCE_PATH"),
        required_env("MY_IDEA_SELF_BUILD_SOURCE_SHA256"),
        required_env("MY_IDEA_SELF_BUILD_COMPILER"),
        required_env("MY_IDEA_SELF_BUILD_COMPILER_REVISION"),
        required_env("MY_IDEA_SELF_BUILD_COMPILER_TARGET"),
        required_env("MY_IDEA_SELF_BUILD_ARTIFACT_PATH"),
        required_env("MY_IDEA_SELF_BUILD_ARTIFACT_SHA256"),
    )
    .unwrap_or_else(|error| panic!("invalid self-build compiler-stage evidence: {error}"));

    verify_compiler_stage_gate(repo_root, &gate)
        .unwrap_or_else(|error| panic!("self-build compiler-stage gate failed: {error}"));

    println!("cargo:rerun-if-changed={}", repo_root.join(gate.source_path()).display());
    println!("cargo:rerun-if-changed={}", repo_root.join(gate.artifact_path()).display());
    println!("cargo:rustc-env=MY_IDEA_SELF_BUILD_PLAN_DIGEST={}", gate.plan_digest());
    println!("cargo:rustc-env=MY_IDEA_SELF_BUILD_SOURCE_REVISION={}", gate.source_revision());
    println!("cargo:rustc-env=MY_IDEA_SELF_BUILD_SOURCE_SHA256={}", gate.source_sha256());
    println!("cargo:rustc-env=MY_IDEA_SELF_BUILD_COMPILER={}", gate.compiler());
    println!("cargo:rustc-env=MY_IDEA_SELF_BUILD_COMPILER_REVISION={}", gate.compiler_revision());
    println!("cargo:rustc-env=MY_IDEA_SELF_BUILD_COMPILER_TARGET={}", gate.compiler_target());
    println!("cargo:rustc-env=MY_IDEA_SELF_BUILD_ARTIFACT_SHA256={}", gate.artifact_sha256());
}

fn main() {

    // Records the exact commit `external/my-lisp` is checked out at when
    // my-idea itself is compiled, as a plain string baked into the binary
    // (never a filesystem path) — repl_process::resolve_or_fetch_my_lisp_binary
    // uses this to `git clone`/checkout the same commit directly from its
    // real GitHub URL on a machine that never had my-idea's own source tree,
    // let alone its submodule, checked out (a packaged release install).
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("src-tauri has a parent repo root");
    let submodule = repo_root.join("external").join("my-lisp");

    verify_self_build_compiler_stage(repo_root);
    tauri_build::build();

    let sha = Command::new("git")
        .arg("-C")
        .arg(&submodule)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default();

    println!("cargo:rustc-env=MY_LISP_PINNED_SHA={sha}");
    println!("cargo:rerun-if-changed={}", submodule.join(".git").display());
}
