//! RED wiring contract for #12: the deterministic plan must be exposed through
//! the real Tauri command surface, while execution remains deferred to #13.

const LIB_RS: &str = include_str!("../src/lib.rs");

#[test]
fn self_build_plan_is_a_registered_tauri_command() {
    assert!(
        LIB_RS.contains("#[tauri::command]\nfn self_build_plan("),
        "lib.rs must expose a self_build_plan Tauri command"
    );

    let handlers = LIB_RS
        .split("tauri::generate_handler![")
        .nth(1)
        .expect("Tauri invoke handler must exist")
        .split("])")
        .next()
        .expect("Tauri invoke handler must terminate");

    assert!(
        handlers.contains("self_build_plan,"),
        "self_build_plan must be registered in tauri::generate_handler!"
    );
}

#[test]
fn self_build_plan_command_is_planning_only() {
    let command = LIB_RS
        .split("#[tauri::command]\nfn self_build_plan(")
        .nth(1)
        .expect("self_build_plan Tauri command must exist")
        .split("#[tauri::command]")
        .next()
        .unwrap_or_default();

    assert!(command.contains("discover_self_build_inputs"));
    assert!(command.contains("build_self_build_plan"));
    assert!(!command.contains("compile_project("));
    assert!(!command.contains("compile_observed("));
    assert!(!command.contains("ProcessService"));
    assert!(!command.contains("start_build("));
}
