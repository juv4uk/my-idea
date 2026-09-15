//! RED contract for #16: the IDE must prove its existing CompilerBridge
//! against a real, pinned CML compiler process, not only a fake executable.
//!
//! This witness is mechanism-only. It does not assert a Lisp result. It
//! verifies canonical source admission, artifact production and compiler
//! provenance from the external process boundary.

#![cfg(unix)]

use my_idea_lib::compiler_bridge::{CompilerBridge, CompilerRequest};
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

#[test]
fn pinned_real_cml_process_produces_an_elf_with_exact_compiler_provenance() {
    let compiler = std::env::var_os("MY_IDEA_CML_COMPILER")
        .map(PathBuf::from)
        .expect("MY_IDEA_CML_COMPILER must point to the pinned real cml-compile binary");
    let expected_revision = std::env::var("MY_IDEA_CML_REVISION")
        .expect("MY_IDEA_CML_REVISION must identify the exact pinned CML checkout");

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("my-idea-real-cml-{nonce}"));
    fs::create_dir_all(&root).unwrap();
    let source = root.join("main.lisp");
    fs::write(&source, "(+ 10 32)\n").unwrap();

    let request = CompilerRequest::new(&source, "x86_64-linux").unwrap();
    let artifact = CompilerBridge::at(&compiler).compile(&request).unwrap();

    assert_eq!(artifact.compiler(), "cml-compile");
    assert_eq!(artifact.compiler_revision(), expected_revision);

    let bytes = fs::read(artifact.path()).expect("real CML must produce the requested artifact");
    assert!(bytes.starts_with(&[0x7f, b'E', b'L', b'F']));

    let _ = fs::remove_file(artifact.path());
    let _ = fs::remove_dir_all(root);
}
