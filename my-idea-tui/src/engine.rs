//! Зв'язок із pinned SENS.
//!
//! TUI **не** містить власного інтерпретатора: він запускає той самий `my-lisp`
//! CLI, який збирається з поточного pinned gitlink `external/sens` (як і GUI-консоль).
//! Шлях до бінарника беремо з `MY_LISP_BIN`; інакше — з `external/sens/target/*`
//! або з `PATH`. Ніякого `PATH`-fallback на чужу ревізію мовчки: якщо бінарника
//! немає — інформативна помилка в консолі.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;

/// Рядок виводу рушія з явним походженням (точні stdout/stderr/system).
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
    stdin: ChildStdin,
    pub rx: Receiver<Line>,
}

impl Engine {
    /// Спроба підняти pinned SENS CLI.
    pub fn spawn(bin: &str) -> std::io::Result<Engine> {
        let mut child = Command::new(bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");

        let (tx, rx) = mpsc::channel::<Line>();

        let tx_out = tx.clone();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().flatten() {
                let _ = tx_out.send(Line { origin: Origin::Stdout, text: line });
            }
        });
        let tx_err = tx.clone();
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().flatten() {
                let _ = tx_err.send(Line { origin: Origin::Stderr, text: line });
            }
        });

        Ok(Engine { child, stdin, rx })
    }

    /// Надіслати рядок рушію (без вигаданої семантики — просто stdin).
    pub fn send_line(&mut self, line: &str) -> std::io::Result<()> {
        self.stdin.write_all(line.as_bytes())?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()
    }

    /// Передати весь вихідний буфер, зберігаючи LF та всі символи.
    pub fn send_source(&mut self, source: &str) -> std::io::Result<()> {
        self.stdin.write_all(source.as_bytes())?;
        if !source.ends_with('\n') { self.stdin.write_all(b"\n")?; }
        self.stdin.flush()
    }

    /// Зупинити процес і дочекатися звільнення його ресурсів.
    pub fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for Engine {
    fn drop(&mut self) { self.stop(); }
}

/// Лише бінарник із закріпленого external/sens; ніякого випадкового PATH.
pub fn resolve_bin() -> Option<String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    let binary = if cfg!(windows) { "my-lisp.exe" } else { "my-lisp" };
    for profile in ["release", "debug"] {
        let path = root.join("external").join("sens").join("target").join(profile).join(binary);
        if path.is_file() { return Some(path.to_string_lossy().into_owned()); }
    }
    None
}
