//! Real-CML integration contract for #54.
//!
//! This is intentionally a normal test, not `#[ignore]`: CI that claims
//! compiler integration coverage must provision the exact CML binary and
//! revision. Local runs without that opt-in simply leave this external-system
//! witness dormant, like other environment-backed integration probes.

#![cfg(unix)]

use my_idea_lib::{
    compiler_bridge::{CompilerBridge, CompilerRequest},
    compiler_build_adapter::CompilerBuildAdapter,
    process_service::{EventSink, ProcessService, RunState},
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{mpsc, Arc},
    time::Duration,
};

fn configured_real_cml() -> Option<(PathBuf, String)> {
    let compiler = match std::env::var("MY_IDEA_CML_TEST_BIN") {
        Ok(value) => PathBuf::from(value),
        Err(_) if std::env::var_os("CI").is_some() => {
            panic!("CI must provision MY_IDEA_CML_TEST_BIN for the real-CML witness")
        }
        Err(_) => return None,
    };
    let revision = std::env::var("MY_IDEA_CML_TEST_REVISION")
        .expect("a configured real CML binary must have an exact pinned revision");
    Some((compiler, revision))
}

fn git_revision(path: &Path) -> String {
    let directory = path.parent().expect("fixture must have a parent directory");
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git must be available for input provenance");
    assert!(output.status.success(), "input provenance must resolve from git");
    String::from_utf8(output.stdout)
        .expect("git revision must be UTF-8")
        .trim()
        .to_owned()
}

#[test]
fn pinned_real_cml_flows_through_production_build_and_run_with_exact_provenance() {
    let Some((cml_bin, expected_cml_revision)) = configured_real_cml() else {
        return;
    };
    assert!(cml_bin.is_file(), "configured real CML binary must exist");

    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/real_cml_pure_arithmetic.lisp");
    assert!(source.is_file(), "canonical tracked Lisp fixture must exist");
    let expected_input_revision = git_revision(&source);

    let adapter = CompilerBuildAdapter::new(CompilerBridge::at(&cml_bin));
    let request = CompilerRequest::new(&source, "x86_64-linux").expect("valid request");
    let compiled = adapter
        .compile_project(&request)
        .expect("the pinned real CML compiler must produce a native artifact");

    let artifact = compiled.artifact();
    assert_eq!(artifact.compiler(), "cml");
    assert_eq!(artifact.compiler_revision(), expected_cml_revision);
    assert_eq!(artifact.input_revision(), expected_input_revision);
    let bytes = fs::read(artifact.path()).expect("compiled artifact must be readable");
    assert!(bytes.starts_with(&[0x7f, b'E', b'L', b'F']));

    eprintln!(
        "real-cml provenance: compiler={} compiler_revision={} input_revision={} artifact={}",
        artifact.compiler(),
        artifact.compiler_revision(),
        artifact.input_revision(),
        artifact.path().display()
    );

    let spec = compiled
        .process_spec()
        .expect("the compiled artifact must yield the production run profile");
    assert_eq!(spec.profile, "compiled-lisp-artifact");

    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let service = ProcessService::default();
    let (sender, receiver) = mpsc::channel();
    let sink: EventSink = Arc::new(move |event| sender.send(event).unwrap());
    let run_id = service
        .start(&workspace, spec, sink)
        .expect("the existing ProcessService must start the compiled artifact");

    let mut events = Vec::new();
    loop {
        let event = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("compiled artifact must reach a terminal run state");
        let terminal = event.state != RunState::Running;
        events.push(event);
        if terminal {
            break;
        }
    }

    assert!(events.iter().all(|event| event.run_id == run_id));
    assert_eq!(events.last().unwrap().state, RunState::Succeeded);

    let _ = fs::remove_file(artifact.path());
}
