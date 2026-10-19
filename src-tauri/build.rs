use std::path::Path;
use std::process::Command;

fn main() {
    tauri_build::build();

    // Records the exact commit `external/sens` is checked out at when
    // my-idea itself is compiled, as a plain string baked into the binary
    // (never a filesystem path) — repl_process::resolve_or_fetch_my_lisp_binary
    // uses this to `git clone`/checkout the same commit directly from its
    // real GitHub URL on a machine that never had my-idea's own source tree,
    // let alone its submodule, checked out (a packaged release install).
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("src-tauri has a parent repo root");
    let submodule = repo_root.join("external").join("sens");

    // Gitlink is the *only* revision authority. A stale submodule checkout
    // must not silently embed a different engine or fetch its sidecar.
    let revision = |working_dir: &Path, args: &[&str]| -> String {
        let output = Command::new("git")
            .current_dir(working_dir)
            .args(args)
            .output()
            .expect("git is required for the exact SENS gitlink");
        assert!(
            output.status.success(),
            "cannot read SENS revision via git {:?}: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("git revision must be UTF-8")
            .trim()
            .to_owned()
    };
    let expected = revision(repo_root, &["rev-parse", "HEAD:external/sens"]);
    let checked_out = revision(&submodule, &["rev-parse", "HEAD"]);
    assert_eq!(
        checked_out, expected,
        "SENS gitlink/checkout differ; git submodule update --init external/sens"
    );

    println!("cargo:rustc-env=MY_LISP_PINNED_SHA={expected}");
    println!("cargo:rerun-if-changed={}", submodule.join(".git").display());
}
