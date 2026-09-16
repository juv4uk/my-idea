use std::path::Path;
use std::process::Command;

fn main() {
    tauri_build::build();

    // Records the exact commit `external/my-lisp` is checked out at when
    // my-idea itself is compiled, as a plain string baked into the binary
    // (never a filesystem path) — repl_process::resolve_or_fetch_my_lisp_binary
    // uses this to `git clone`/checkout the same commit directly from its
    // real GitHub URL on a machine that never had my-idea's own source tree,
    // let alone its submodule, checked out (a packaged release install).
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("src-tauri has a parent repo root");
    let submodule = repo_root.join("external").join("my-lisp");

    let sha = Command::new("git")
        .arg("-C")
        .arg(&submodule)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default();

    println!("cargo:rustc-env=MY_LISP_PINNED_SHA={sha}");
    println!("cargo:rerun-if-changed={}", submodule.join(".git").display());
}
