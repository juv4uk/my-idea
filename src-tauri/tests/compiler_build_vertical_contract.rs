//! RED contract for #16 vertical slice: a canonical .lisp source must flow
//! through one authoritative compiler process, preserve compiler observations,
//! and enter the existing Build/Run process service through a fixed profile.

#![cfg(unix)]

use my_idea_lib::{
    compiler_bridge::{CompilerBridge, CompilerRequest, DiagnosticStream},
    compiler_build_adapter::CompilerBuildAdapter,
    process_service::{EventSink, ProcessService, RunState},
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    sync::{mpsc, Arc},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[test]
fn authoritative_compiler_output_flows_into_fixed_build_profile() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("my-idea-compiler-vertical-{nonce}"));
    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace).unwrap();

    let source = workspace.join(format!("hello-{nonce}.lisp"));
    fs::write(&source, "(+ 40 2)\n").unwrap();

    let compiler = root.join("fake-cml");
    fs::write(
        &compiler,
        r#"#!/bin/sh
set -eu
test "$1" = "x86-elf"
printf '#!/bin/sh\nprintf "artifact-ran\\n"\n' > "$3"
chmod +x "$3"
printf 'compiler-stdout\n'
printf 'compiler-stderr\n' >&2
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&compiler).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&compiler, permissions).unwrap();

    let request = CompilerRequest::new(&source, "x86_64-linux").unwrap();
    let adapter = CompilerBuildAdapter::new(CompilerBridge::at(&compiler));
    let compiled = adapter.compile_project(&request).unwrap();

    assert!(compiled.artifact().path().is_file());
    assert!(compiled.diagnostics().iter().any(|diagnostic| {
        diagnostic.stream() == DiagnosticStream::Stdout
            && diagnostic.message() == "compiler-stdout"
    }));
    assert!(compiled.diagnostics().iter().any(|diagnostic| {
        diagnostic.stream() == DiagnosticStream::Stderr
            && diagnostic.message() == "compiler-stderr"
    }));

    let spec = compiled.process_spec().unwrap();
    assert_eq!(spec.profile, "compiled-lisp-artifact");
    assert!(spec.executable.is_absolute());
    assert_eq!(spec.executable, compiled.artifact().path().canonicalize().unwrap());
    assert!(spec.args.is_empty());

    let service = ProcessService::default();
    let (sender, receiver) = mpsc::channel();
    let sink: EventSink = Arc::new(move |event| sender.send(event).unwrap());
    let run_id = service.start(&workspace, spec, sink).unwrap();

    let mut events = Vec::new();
    loop {
        let event = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        let terminal = event.state != RunState::Running;
        events.push(event);
        if terminal {
            break;
        }
    }

    assert!(events.iter().all(|event| event.run_id == run_id));
    assert!(events.iter().any(|event| event.line == "artifact-ran"));
    assert_eq!(events.last().unwrap().state, RunState::Succeeded);

    let artifact = compiled.artifact().path();
    let _ = fs::remove_file(artifact);
    let _ = fs::remove_dir_all(root);
}
