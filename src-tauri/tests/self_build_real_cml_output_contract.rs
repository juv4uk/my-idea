//! Physical #13 witness: the exact pinned CML provisioned by CI must compile
//! the tracked self-build Lisp input to the exact artifact declared by
//! self-build-plan-v1.

#![cfg(unix)]

use my_idea_lib::{
    compiler_bridge::{CompilerBridge, CompilerRequest},
    compiler_build_adapter::CompilerBuildAdapter,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn configured_real_cml() -> Option<(PathBuf, String)> {
    let compiler = match std::env::var("MY_IDEA_CML_TEST_BIN") {
        Ok(value) => PathBuf::from(value),
        Err(_) if std::env::var_os("CI").is_some() => {
            panic!("CI must provision MY_IDEA_CML_TEST_BIN for the self-build compiler witness")
        }
        Err(_) => return None,
    };
    let revision = std::env::var("MY_IDEA_CML_TEST_REVISION")
        .expect("provisioned CML must have an exact revision");
    Some((compiler, revision))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri must have a repository root parent")
        .to_path_buf()
}

fn git_head(path: &Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git provenance probe must start");
    assert!(output.status.success(), "git HEAD must resolve");
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

#[test]
fn pinned_cml_produces_exact_self_build_plan_artifact() {
    let Some((cml, expected_compiler_revision)) = configured_real_cml() else {
        return;
    };

    let root = repo_root();
    let source = root.join("self-build/my-idea.lisp");
    let output = root.join("target/self-build/my-idea-x86_64-linux");
    assert!(source.is_file(), "tracked self-build Lisp input must exist");

    let adapter = CompilerBuildAdapter::new(CompilerBridge::at(&cml));
    let request = CompilerRequest::new(&source, "x86_64-linux").unwrap();
    let compiled = adapter
        .compile_project_to(&request, &output)
        .expect("pinned CML must compile the self-build input to the planned output");

    let artifact = compiled.artifact();
    assert_eq!(artifact.path(), output);
    assert_eq!(artifact.compiler(), "cml-compile");
    assert_eq!(artifact.compiler_revision(), expected_compiler_revision);
    assert_eq!(artifact.input_revision(), git_head(&root));

    let bytes = fs::read(artifact.path()).expect("self-build compiler artifact must be readable");
    assert!(bytes.starts_with(&[0x7f, b'E', b'L', b'F']));

    eprintln!(
        "self-build-cml provenance: compiler={} compiler_revision={} input_revision={} artifact={}",
        artifact.compiler(),
        artifact.compiler_revision(),
        artifact.input_revision(),
        artifact.path().display()
    );

    let _ = fs::remove_file(artifact.path());
}
