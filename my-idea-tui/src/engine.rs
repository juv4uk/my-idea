//! Хостовий процес для точного SENS CLI з `external/sens`.
//! Файл виконується через аргумент CLI; інтерактивні рядки — через REPL.
//! Жодної Lisp-семантики, декодування .sens або PATH fallback у TUI.

use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;

#[derive(Clone, Copy, PartialEq)]
pub enum Origin {
    Stdout,
    Stderr,
    System,
}

#[derive(Clone)]
pub struct Line {
    pub origin: Origin,
    pub text: String,
}

pub struct Engine {
    child: Child,
    stdin: Option<ChildStdin>,
    pub rx: Receiver<Line>,
}

impl Engine {
    /// Інтерактивний SENS CLI, із живим stdin лише для REPL.
    pub fn spawn_repl(bin: &Path) -> io::Result<Self> {
        let mut command = Command::new(bin);
        command.stdin(Stdio::piped());
        Self::spawn(command)
    }

    /// Точний file-mode: sens <file>, а НЕ рядки файла в REPL.
    pub fn spawn_file(bin: &Path, source: &Path) -> io::Result<Self> {
        let mut command = Command::new(bin);
        command.arg(source).stdin(Stdio::null());
        Self::spawn(command)
    }

    fn spawn(mut command: Command) -> io::Result<Self> {
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let (tx, rx) = mpsc::channel();

        let stdout_tx = tx.clone();
        thread::spawn(move || {
            for result in BufReader::new(stdout).lines() {
                match result {
                    Ok(text) => { let _ = stdout_tx.send(Line { origin: Origin::Stdout, text }); }
                    Err(error) => {
                        let _ = stdout_tx.send(Line { origin: Origin::Stderr, text: format!("stdout read: {error}") });
                        break;
                    }
                }
            }
        });
        thread::spawn(move || {
            for result in BufReader::new(stderr).lines() {
                match result {
                    Ok(text) => { let _ = tx.send(Line { origin: Origin::Stderr, text }); }
                    Err(error) => {
                        let _ = tx.send(Line { origin: Origin::Stderr, text: format!("stderr read: {error}") });
                        break;
                    }
                }
            }
        });
        Ok(Self { child, stdin, rx })
    }

    pub fn send_line(&mut self, line: &str) -> io::Result<()> {
        let stdin = self.stdin.as_mut().ok_or_else(|| io::Error::new(
            io::ErrorKind::BrokenPipe, "process is not an interactive REPL"
        ))?;
        stdin.write_all(line.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    pub fn stop(&mut self) {
        // Завершений процес повторно вбивати не потрібно.
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Лише локальна збірка з поточного pinned підмодуля — ніякого PATH,
/// MY_LISP_BIN або завантаження іншої ревізії.
pub fn resolve_bin() -> Option<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    if !root.join("external/sens/Cargo.toml").is_file() {
        return None;
    }
    for profile in ["release", "debug"] {
        for name in if cfg!(windows) { ["sens.exe", "my-lisp.exe"] } else { ["sens", "my-lisp"] } {
            let bin = root.join("external/sens/target").join(profile).join(name);
            if bin.is_file() {
                return Some(bin);
            }
        }
    }
    None
}
