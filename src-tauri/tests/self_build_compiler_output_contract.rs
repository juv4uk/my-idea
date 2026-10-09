//! RED contract for #13: the self-build compiler stage must be able to
//! request the exact output path named by self-build-plan-v1 without changing
//! the generic IDE compiler path.

use my_idea_lib::{
    compiler_bridge::{CompilerBridge, CompilerRequest},
    compiler_build_adapter::CompilerBuildAdapter,
};
use std::path::PathBuf;

#[test]
fn compiler_bridge_exposes_explicit_output_path_for_self_build() {
    let bridge = CompilerBridge::at("/definitely/missing/cml-compile");
    let request = CompilerRequest::new("self-build/my-idea.lisp", "x86_64-linux").unwrap();
    let output = PathBuf::from("target/self-build/my-idea-x86_64-linux");

    let failure = bridge
        .compile_observed_to(&request, &output)
        .expect_err("missing compiler must still fail before producing an artifact");

    assert!(failure.to_string().contains("compiler"));
}

#[test]
fn compiler_build_adapter_exposes_the_same_explicit_output_boundary() {
    let adapter = CompilerBuildAdapter::new(CompilerBridge::at(
        "/definitely/missing/cml-compile",
    ));
    let request = CompilerRequest::new("self-build/my-idea.lisp", "x86_64-linux").unwrap();
    let output = PathBuf::from("target/self-build/my-idea-x86_64-linux");

    let failure = adapter
        .compile_project_to(&request, &output)
        .expect_err("adapter must delegate the exact output path to the compiler bridge");

    assert!(failure.to_string().contains("compiler"));
}
