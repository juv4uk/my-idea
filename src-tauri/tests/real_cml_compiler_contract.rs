//! Empirical proof that #16's vertical slice (`CompilerBridge` +
//! `CompilerBuildAdapter`) works against the *real* `cml` compiler binary,
//! not just `compiler_build_vertical_contract.rs`'s fake-script stand-in.
//! Ignored by default (needs a real, built `cml`) the same way
//! `lsp_client.rs`'s real-server tests are — set `MY_IDEA_CML_TEST_BIN` to
//! a `cml` binary built from `cargo build --release --bin cml` in the
//! `cml` repo to run it.

use my_idea_lib::compiler_bridge::{CompilerBridge, CompilerRequest};
use my_idea_lib::compiler_build_adapter::CompilerBuildAdapter;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
#[ignore = "requires MY_IDEA_CML_TEST_BIN pointing at a real built cml binary"]
fn real_cml_compiles_pure_arithmetic_to_a_runnable_native_elf() {
    let cml_bin = std::env::var("MY_IDEA_CML_TEST_BIN")
        .expect("set MY_IDEA_CML_TEST_BIN to a real cml binary to run this test");

    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("my-idea-real-cml-contract-{nonce}"));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    let source = dir.join("hello.lisp");
    fs::write(&source, "(+ 40 2)\n").expect("source should be written");

    let adapter = CompilerBuildAdapter::new(CompilerBridge::at(PathBuf::from(cml_bin)));
    let request = CompilerRequest::new(&source, "x86_64-linux").expect("valid request");
    let compiled = adapter
        .compile_project(&request)
        .expect("the real cml compiler should compile pure arithmetic to a native artifact");

    let spec = compiled
        .process_spec()
        .expect("a successfully compiled artifact should yield a runnable process spec");
    assert_eq!(spec.profile, "compiled-lisp-artifact");

    let status = Command::new(&spec.executable)
        .status()
        .expect("the compiled native ELF artifact should actually run");
    assert!(status.success(), "the compiled artifact should exit successfully");

    let _ = fs::remove_dir_all(&dir);
}
