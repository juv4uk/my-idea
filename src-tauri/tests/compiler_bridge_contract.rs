//! RED contract for #16: `my-idea` treats our compiler/CML as the
//! authoritative project compiler, while the IDE remains a mechanism-only
//! client of that compiler.

use my_idea_lib::compiler_bridge::{
    CompilerArtifact, CompilerBridge, CompilerDiagnostic, CompilerRequest, DiagnosticStream,
};
use std::path::PathBuf;

#[test]
fn compiler_request_is_canonical_lisp_source_plus_explicit_target() {
    let request = CompilerRequest::new("examples/hello.lisp", "x86_64-linux").unwrap();

    assert_eq!(request.source(), PathBuf::from("examples/hello.lisp"));
    assert_eq!(request.target(), "x86_64-linux");
}

#[test]
fn compiler_diagnostics_are_transport_data_not_ide_semantics() {
    let diagnostic = CompilerDiagnostic::new(
        DiagnosticStream::Stderr,
        "src/main.lisp",
        Some(7),
        Some(12),
        "unexpected form",
    );

    assert_eq!(diagnostic.stream(), DiagnosticStream::Stderr);
    assert_eq!(diagnostic.path(), "src/main.lisp");
    assert_eq!(diagnostic.line(), Some(7));
    assert_eq!(diagnostic.column(), Some(12));
    assert_eq!(diagnostic.message(), "unexpected form");
}

#[test]
fn successful_compilation_carries_artifact_and_compiler_provenance() {
    let artifact = CompilerArtifact::new(
        "target/my-idea/hello",
        "cml",
        "0123456789abcdef0123456789abcdef01234567",
        "89abcdef0123456789abcdef0123456789abcdef",
    )
    .unwrap();

    assert_eq!(artifact.path(), PathBuf::from("target/my-idea/hello"));
    assert_eq!(artifact.compiler(), "cml");
    assert_eq!(artifact.compiler_revision(), "0123456789abcdef0123456789abcdef01234567");
    assert_eq!(artifact.input_revision(), "89abcdef0123456789abcdef0123456789abcdef");
}

#[test]
fn missing_authoritative_compiler_fails_closed() {
    let bridge = CompilerBridge::at("/definitely/missing/my-lisp-compiler");
    let request = CompilerRequest::new("examples/hello.lisp", "x86_64-linux").unwrap();

    let error = bridge.compile(&request).unwrap_err();
    assert!(error.to_string().contains("compiler"));
}
