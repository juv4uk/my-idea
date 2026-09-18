//! RED contract for #12 (IDE-SELF-BUILD-1): my-idea must describe the
//! next self-build deterministically before #13 is allowed to execute it.

use my_idea_lib::self_build::{
    build_self_build_plan, canonical_plan_json, SelfBuildInputs,
};

fn inputs() -> SelfBuildInputs {
    SelfBuildInputs::new(
        "1111111111111111111111111111111111111111",
        true,
        "2222222222222222222222222222222222222222",
        "cml-compile",
        "3333333333333333333333333333333333333333",
        "x86_64-linux",
        "rustc 1.93.0",
        "cargo 1.93.0",
        "1.3.8",
        "2.11.4",
    )
}

#[test]
fn same_inputs_produce_byte_identical_plan_and_digest() {
    let first = build_self_build_plan(inputs()).expect("fixed provenance must produce a plan");
    let second = build_self_build_plan(inputs()).expect("same provenance must produce a plan");

    assert_eq!(
        canonical_plan_json(&first).unwrap(),
        canonical_plan_json(&second).unwrap()
    );
    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.digest().len(), 64);
    assert!(first.digest().chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn changing_compiler_revision_changes_digest() {
    let first = build_self_build_plan(inputs()).unwrap();
    let changed = SelfBuildInputs::new(
        "1111111111111111111111111111111111111111",
        true,
        "2222222222222222222222222222222222222222",
        "cml-compile",
        "4444444444444444444444444444444444444444",
        "x86_64-linux",
        "rustc 1.93.0",
        "cargo 1.93.0",
        "1.3.8",
        "2.11.4",
    );

    assert_ne!(
        first.digest(),
        build_self_build_plan(changed).unwrap().digest()
    );
}

#[test]
fn plan_records_exact_stage_order_and_tauri_bootstrap_boundary() {
    let plan = build_self_build_plan(inputs()).unwrap();

    assert_eq!(plan.stage_names(), ["compiler", "frontend", "platform"]);
    assert_eq!(plan.compiler_target(), "x86_64-linux");
    assert!(plan.platform_stage().bootstrap_platform());

    let commands = plan.stage_commands();
    assert_eq!(
        commands[0],
        (
            "cml-compile",
            vec![
                "x86-elf",
                "self-build/my-idea.lisp",
                "target/self-build/my-idea-x86_64-linux",
            ],
        )
    );
    assert_eq!(commands[1], ("bun", vec!["run", "build"]));
    assert_eq!(commands[2], ("bun", vec!["run", "tauri", "build"]));
}

#[test]
fn plan_exposes_exact_source_runtime_and_compiler_revisions() {
    let plan = build_self_build_plan(inputs()).unwrap();

    assert_eq!(
        plan.source_revision(),
        "1111111111111111111111111111111111111111"
    );
    assert_eq!(
        plan.my_lisp_revision(),
        "2222222222222222222222222222222222222222"
    );
    assert_eq!(
        plan.compiler_revision(),
        "3333333333333333333333333333333333333333"
    );
}

#[test]
fn dirty_source_tree_fails_closed() {
    let dirty = SelfBuildInputs::new(
        "1111111111111111111111111111111111111111",
        false,
        "2222222222222222222222222222222222222222",
        "cml-compile",
        "3333333333333333333333333333333333333333",
        "x86_64-linux",
        "rustc 1.93.0",
        "cargo 1.93.0",
        "1.3.8",
        "2.11.4",
    );

    assert!(build_self_build_plan(dirty)
        .unwrap_err()
        .to_string()
        .contains("dirty"));
}

#[test]
fn malformed_or_unavailable_provenance_fails_closed() {
    let bad_source = SelfBuildInputs::new(
        "short",
        true,
        "2222222222222222222222222222222222222222",
        "cml-compile",
        "3333333333333333333333333333333333333333",
        "x86_64-linux",
        "rustc 1.93.0",
        "cargo 1.93.0",
        "1.3.8",
        "2.11.4",
    );
    assert!(build_self_build_plan(bad_source).is_err());

    let unavailable_compiler = SelfBuildInputs::new(
        "1111111111111111111111111111111111111111",
        true,
        "2222222222222222222222222222222222222222",
        "cml-compile",
        "unavailable",
        "x86_64-linux",
        "rustc 1.93.0",
        "cargo 1.93.0",
        "1.3.8",
        "2.11.4",
    );
    assert!(build_self_build_plan(unavailable_compiler).is_err());

    let missing_bun = SelfBuildInputs::new(
        "1111111111111111111111111111111111111111",
        true,
        "2222222222222222222222222222222222222222",
        "cml-compile",
        "3333333333333333333333333333333333333333",
        "x86_64-linux",
        "rustc 1.93.0",
        "cargo 1.93.0",
        "",
        "2.11.4",
    );
    assert!(build_self_build_plan(missing_bun).is_err());
}

#[test]
fn unsupported_target_fails_closed() {
    let unsupported = SelfBuildInputs::new(
        "1111111111111111111111111111111111111111",
        true,
        "2222222222222222222222222222222222222222",
        "cml-compile",
        "3333333333333333333333333333333333333333",
        "aarch64-linux",
        "rustc 1.93.0",
        "cargo 1.93.0",
        "1.3.8",
        "2.11.4",
    );

    assert!(build_self_build_plan(unsupported)
        .unwrap_err()
        .to_string()
        .contains("x86_64-linux"));
}
