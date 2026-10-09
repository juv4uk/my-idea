use my_idea_lib::{
    compiler_bridge::resolve_cml_executable,
    self_build::{
        build_self_build_plan, discover_self_build_inputs, prepare_compiler_stage,
        verify_compiler_stage,
    },
};
use std::{env, path::PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("self-build compiler stage failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "src-tauri must have a repository root parent".to_string())?
        .to_path_buf();
    let explicit = env::var_os("MY_IDEA_CML_BIN").map(PathBuf::from);
    let cml = resolve_cml_executable(&repo_root, explicit)?;

    let inputs = discover_self_build_inputs(&repo_root, &cml)
        .map_err(|error| error.to_string())?;
    let plan = build_self_build_plan(inputs)
        .map_err(|error| error.to_string())?;
    let evidence = prepare_compiler_stage(&repo_root, &cml, &plan)
        .map_err(|error| error.to_string())?;
    verify_compiler_stage(&repo_root, &plan, &evidence)
        .map_err(|error| error.to_string())?;

    println!(
        "{}",
        serde_json::to_string(&evidence)
            .map_err(|error| format!("compiler-stage evidence serialization failed: {error}"))?
    );
    Ok(())
}
