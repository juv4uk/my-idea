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

    /// Зупинити реальне виконання (Stop у вертикалі).
    pub fn stop(&mut self) {
        let _ = self.child.kill();
    }
}

/// Знайти бінарник рушія: `MY_LISP_BIN` → `external/sens/target/{debug,release}/my-lisp` → `PATH`.
pub fn resolve_bin() -> Option<String> {
    if let Ok(p) = std::env::var("MY_LISP_BIN") {
        if !p.is_empty() {
            return Some(p);
        }
    }
    for profile in ["release", "debug"] {
        let p = format!("external/sens/target/{profile}/my-lisp");
        if std::path::Path::new(&p).exists() {
            return Some(p);
        }
    }
    // Останній шанс — PATH; явно, не мовчки.
    if which("my-lisp").is_some() {
        return Some("my-lisp".to_string());
    }
    None
}

fn which(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(name);
        if cand.is_file() {
            return Some(cand.to_string_lossy().into_owned());
        }
    }
    None
}
