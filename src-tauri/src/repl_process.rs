//! Wraps the *real* `my-lisp` CLI REPL binary as a live child process,
//! instead of reimplementing any of its logic (surfaces, banner,
//! presentation) inside my-idea. The binary is built on demand from the
//! `external/my-lisp` git submodule, so it is always exactly the pinned
//! commit's own REPL — a submodule bump carries every surface fix or
//! extension automatically, with zero duplicated Rust here.
//!
//! Обгортає *справжній* CLI REPL-бінарник `my-lisp` як живий дочірній
//! процес, замість того щоб переносити його логіку (поверхні, банер,
//! презентацію) в код my-idea. Бінарник збирається за потреби з git
//! підмодуля `external/my-lisp`, тож це завжди саме той REPL, що
//! відповідає закріпленому коміту — оновлення підмодуля автоматично
//! переносить будь-яке виправлення чи розширення поверхні без жодного
//! дубльованого Rust-коду тут.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplProcessStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug)]
pub struct ReplProcessLine {
    pub stream: ReplProcessStream,
    pub line: String,
}

/// A persistent, interactive child process fed one line at a time through
/// its stdin, with stdout/stderr lines forwarded to a callback as they
/// arrive.
pub struct ReplProcess {
    child: Mutex<Child>,
    stdin: Mutex<std::process::ChildStdin>,
}

impl ReplProcess {
    pub fn spawn(
        executable: &Path,
        args: &[&str],
        on_line: impl Fn(ReplProcessLine) + Send + Sync + 'static,
    ) -> Result<Self, String> {
        let mut child = Command::new(executable)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("failed to start {}: {error}", executable.display()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "child process has no stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "child process has no stdout".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "child process has no stderr".to_string())?;

        let on_line: Arc<dyn Fn(ReplProcessLine) + Send + Sync> = Arc::new(on_line);
        spawn_line_forwarder(stdout, ReplProcessStream::Stdout, on_line.clone());
        spawn_line_forwarder(stderr, ReplProcessStream::Stderr, on_line);

        Ok(Self {
            child: Mutex::new(child),
            stdin: Mutex::new(stdin),
        })
    }

    /// Writes one line to the process's stdin, appending the newline that
    /// terminates it (mirroring how a real terminal submits a typed line).
    pub fn write_line(&self, line: &str) -> Result<(), String> {
        let mut stdin = self
            .stdin
            .lock()
            .map_err(|_| "repl process stdin poisoned".to_string())?;
        writeln!(stdin, "{line}").map_err(|error| format!("failed to write to repl process: {error}"))?;
        stdin
            .flush()
            .map_err(|error| format!("failed to flush repl process stdin: {error}"))
    }

    pub fn kill(&self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
    }
}

fn spawn_line_forwarder<R: Read + Send + 'static>(
    reader: R,
    stream: ReplProcessStream,
    on_line: Arc<dyn Fn(ReplProcessLine) + Send + Sync>,
) {
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines().map_while(Result::ok) {
            on_line(ReplProcessLine { stream, line });
        }
    });
}

/// Resolves the path to the real `my-lisp` CLI binary, building it from the
/// `external/my-lisp` git submodule if it isn't already built (cargo's own
/// incremental cache makes every call after the first effectively free).
/// `repo_root` is my-idea's own repository root (the parent of `src-tauri`).
///
/// Only usable when that repo root actually has the submodule checked out —
/// true for a source/dev checkout, never true for a packaged install (see
/// `resolve_or_fetch_my_lisp_binary`, which this repo's console command
/// actually calls).
pub fn resolve_my_lisp_binary(repo_root: &Path) -> Result<PathBuf, String> {
    let submodule = repo_root.join("external").join("my-lisp");
    if !submodule.join("Cargo.toml").exists() {
        return Err(format!(
            "external/my-lisp submodule not checked out at {} — run `git submodule update --init`",
            submodule.display()
        ));
    }
    build_my_lisp_cli(&submodule)
}

/// The exact `my-lisp` commit `external/my-lisp` was checked out at when
/// this my-idea binary was compiled (`src-tauri/build.rs`) — a plain string
/// identifier, never a filesystem path, so it stays meaningful on whatever
/// machine later runs the compiled binary. Empty if the submodule was
/// somehow missing at compile time.
pub fn my_lisp_pinned_sha() -> &'static str {
    env!("MY_LISP_PINNED_SHA")
}

/// The real upstream URL `resolve_or_fetch_my_lisp_binary` clones when no
/// local submodule checkout is available.
pub const MY_LISP_GIT_URL: &str = "https://github.com/juv4uk/my-lisp.git";

/// Resolves the my-lisp CLI binary via the local `external/my-lisp`
/// submodule when a source/dev checkout of my-idea provides one (fast path,
/// no network) — otherwise falls back to cloning the exact pinned commit
/// directly from its real GitHub URL into `cache_dir` and building it
/// there. The fallback is what makes a *packaged* my-idea install able to
/// run the console at all: an installed app's own directory never has
/// my-idea's source tree, let alone its submodule, on disk — the only
/// thing that can possibly be "checked out" on that machine is a git
/// reference (URL + pinned SHA), never a filesystem path baked in when the
/// binary was compiled somewhere else entirely.
pub fn resolve_or_fetch_my_lisp_binary(repo_root: &Path, cache_dir: &Path) -> Result<PathBuf, String> {
    let local_submodule = repo_root.join("external").join("my-lisp");
    if local_submodule.join("Cargo.toml").exists() {
        return build_my_lisp_cli(&local_submodule);
    }
    fetch_and_build_my_lisp(cache_dir)
}

fn fetch_and_build_my_lisp(cache_dir: &Path) -> Result<PathBuf, String> {
    let pinned_sha = my_lisp_pinned_sha();
    if pinned_sha.is_empty() {
        return Err(
            "my-lisp's pinned commit was not recorded when my-idea was built (external/my-lisp \
             submodule was missing at compile time) — the REPL console cannot resolve which \
             my-lisp to run"
                .to_string(),
        );
    }

    std::fs::create_dir_all(cache_dir)
        .map_err(|error| format!("failed to create {}: {error}", cache_dir.display()))?;
    let checkout = cache_dir.join("my-lisp");

    if !checkout.join(".git").exists() {
        run_git(None, &["clone", MY_LISP_GIT_URL, &checkout.to_string_lossy()])?;
    }

    let already_on_pinned_commit = Command::new("git")
        .arg("-C")
        .arg(&checkout)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == pinned_sha)
        .unwrap_or(false);

    if !already_on_pinned_commit {
        run_git(Some(&checkout), &["fetch", "--depth", "1", "origin", pinned_sha])?;
        run_git(Some(&checkout), &["checkout", "--detach", pinned_sha])?;
    }

    build_my_lisp_cli(&checkout)
}

fn run_git(cwd: Option<&Path>, args: &[&str]) -> Result<(), String> {
    let mut command = Command::new("git");
    if let Some(dir) = cwd {
        command.arg("-C").arg(dir);
    }
    let status = command
        .args(args)
        .status()
        .map_err(|error| format!("failed to run git {args:?}: {error}"))?;
    if !status.success() {
        return Err(format!("git {args:?} failed"));
    }
    Ok(())
}

fn build_my_lisp_cli(my_lisp_checkout: &Path) -> Result<PathBuf, String> {
    let manifest = my_lisp_checkout.join("Cargo.toml");
    let target_dir = my_lisp_checkout.join("target");
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "my-lisp-cli", "--bin", "my-lisp"])
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--target-dir")
        .arg(&target_dir)
        .status()
        .map_err(|error| format!("failed to run cargo build for the my-lisp CLI: {error}"))?;
    if !status.success() {
        return Err("cargo build for the my-lisp CLI failed".to_string());
    }

    let binary_name = if cfg!(windows) { "my-lisp.exe" } else { "my-lisp" };
    let binary = target_dir.join("release").join(binary_name);
    if !binary.exists() {
        return Err(format!(
            "expected the my-lisp binary at {} after a successful build",
            binary.display()
        ));
    }
    Ok(binary)
}

/// my-idea's own repository root, resolved from the crate's build-time
/// manifest directory (`src-tauri`'s parent).
pub fn my_idea_repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent repo root")
        .to_path_buf()
}
