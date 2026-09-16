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
pub fn resolve_my_lisp_binary(repo_root: &Path) -> Result<PathBuf, String> {
    let submodule = repo_root.join("external").join("my-lisp");
    let manifest = submodule.join("Cargo.toml");
    if !manifest.exists() {
        return Err(format!(
            "external/my-lisp submodule not checked out at {} — run `git submodule update --init`",
            submodule.display()
        ));
    }

    let target_dir = submodule.join("target");
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "my-lisp-cli", "--bin", "my-lisp"])
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--target-dir")
        .arg(&target_dir)
        .status()
        .map_err(|error| format!("failed to run cargo build for the my-lisp CLI: {error}"))?;
    if !status.success() {
        return Err(
            "cargo build for the my-lisp CLI (external/my-lisp submodule) failed".to_string(),
        );
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
