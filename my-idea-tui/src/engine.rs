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
    readers: Vec<thread::JoinHandle<()>>, 
}

impl Engine {
    /// Інтерактивний SENS CLI, із живим stdin лише для REPL.
    pub fn spawn_repl(bin: &Path, workspace: &Path) -> io::Result<Self> {
        let mut command = Command::new(bin);
        command.arg("--surface=укр").current_dir(workspace).stdin(Stdio::piped());
        Self::spawn(command)
    }

    /// Точний file-mode: sens <file>, а НЕ рядки файла в REPL.
    pub fn spawn_file(bin: &Path, source: &Path, workspace: &Path) -> io::Result<Self> {
        let mut command = Command::new(bin);
        command.current_dir(workspace).arg(source).stdin(Stdio::null());
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
        let stdout_reader = thread::spawn(move || {
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
        let stderr_reader = thread::spawn(move || {
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
        Ok(Self { child, stdin, rx, readers: vec![stdout_reader, stderr_reader] })
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

    /// Після завершення процесу дочитати весь реальний stdout/stderr перед
    /// публікацією exit-status. Інакше швидкий повторний F5 губить хвіст виводу.
    pub fn finish_readers(&mut self) {
        for reader in self.readers.drain(..) {
            let _ = reader.join();
        }
    }

    pub fn stop(&mut self) {
        // Завершений процес повторно вбивати не потрібно.
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        self.finish_readers();
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Результат резолюції містить точну ревізію підмодуля, а не лише назву файла.
pub struct ResolvedBin {
    pub path: PathBuf,
    pub pinned_sha: String,
}

fn repository_root(start: &Path) -> Option<PathBuf> {
    start.ancestors().find(|candidate| {
        candidate.join(".gitmodules").is_file()
            && candidate.join("my-idea-tui/Cargo.toml").is_file()
    }).map(Path::to_path_buf)
}

fn revision(root: &Path, reference: &str) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C").arg(root)
        .arg("rev-parse").arg(reference)
        .output()
        .map_err(|error| format!("не вдалося запустити git: {error}"))?;
    let sha = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || sha.len() != 40 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("git не визначив ревізію {reference} у {}", root.display()));
    }
    Ok(sha)
}

/// Шукати від фактичного EXE, потім від робочого каталогу — ніколи від
/// CARGO_MANIFEST_DIR, який містить шлях машини, що компілювала TUI.
/// Ревізія checkout зовнішнього SENS мусить збігатися з gitlink HEAD.
pub fn resolve_bin() -> Result<ResolvedBin, String> {
    let root = std::env::current_exe().ok()
        .and_then(|exe| repository_root(&exe))
        .or_else(|| std::env::current_dir().ok().and_then(|cwd| repository_root(&cwd)))
        .ok_or_else(|| "не знайдено checkout my-idea з .gitmodules поруч із TUI".to_string())?;
    let submodule = root.join("external/sens");
    if !submodule.join("Cargo.toml").is_file() {
        return Err("закріплений external/sens не ініціалізовано (git submodule update --init external/sens)".into());
    }
    let pinned_sha = revision(&root, "HEAD:external/sens")?;
    let checkout_sha = revision(&submodule, "HEAD")?;
    if checkout_sha != pinned_sha {
        return Err(format!(
            "SENS checkout {} не відповідає pinned gitlink {}",
            checkout_sha, pinned_sha
        ));
    }
    let binary = if cfg!(windows) { ["sens.exe", "my-lisp.exe"] } else { ["sens", "my-lisp"] };
    for profile in ["release", "debug"] {
        for name in binary {
            let path = submodule.join("target").join(profile).join(name);
            if path.is_file() {
                return Ok(ResolvedBin { path, pinned_sha });
            }
        }
    }
    Err(format!(
        "немає зібраного CLI в external/sens/target (gitlink {pinned_sha}); зберіть SENS із цієї ревізії"
    ))
}

#[cfg(test)]
mod repository_tests {
    use super::repository_root;
    use std::path::Path;

    #[test]
    fn finds_repository_above_tui_binary_location() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let target = root.join("my-idea-tui/target/release/my-idea-tui");
        assert_eq!(repository_root(&target).as_deref(), Some(root));
    }
}

// Цей тест навмисно не запускається від звичайного cargo test: для нього
// потрібна реальна збірка SENS із поточного gitlink. TUI workflow запускає
// його явно, після перевірки точного SHA підмодуля.
#[cfg(test)]
mod pinned_file_mode_smoke {
    use super::{Engine, Origin};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::ExitStatus;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NONCE: AtomicU64 = AtomicU64::new(0);

    fn run_file(binary: &Path, file: &Path) -> (ExitStatus, String, String) {
        let mut engine = Engine::spawn_file(binary, file, file.parent().expect("workspace файла")).expect("запустити реальний SENS");
        let status = engine.child.wait().expect("дочекатися SENS");
        engine.finish_readers(); // прочитати stdout/stderr до фіксації результату
        let mut stdout = String::new();
        let mut stderr = String::new();
        for line in engine.rx.try_iter() {
            match line.origin {
                Origin::Stdout => { stdout.push_str(&line.text); stdout.push('\n'); }
                Origin::Stderr => { stderr.push_str(&line.text); stderr.push('\n'); }
                Origin::System => panic!("SENS CLI не створює TUI-повідомлень"),
            }
        }
        (status, stdout, stderr)
    }

    /// Перевірка не через mock: точно той самий CLI з gitlink, який запускає TUI.
    /// У TUI мова uk|de|en — інтерфейс; --surface=укр керує лише справжнім SENS.
    #[test]
    #[ignore = "реальний SENS_PINNED_CLI потрібен; CI перевіряє разом із file-mode"]
    fn pinned_ukr_repl_smoke() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let expected = root.join("external/sens/target/release/my-lisp")
            .canonicalize().expect("бінарник із pinned external/sens");
        let binary = PathBuf::from(std::env::var_os("SENS_PINNED_CLI")
            .expect("SENS_PINNED_CLI вказує на точний gitlink"));
        assert_eq!(binary.canonicalize().expect("CLI"), expected,
            "не виконувати сторонній CLI");

        let mut engine = Engine::spawn_repl(&binary, root)
            .expect("SENS REPL --surface=укр має запуститися");
        engine.send_line("(атом? 'мама)").expect("надсилання української форми");
        // Закрити stdin: rustyline/REPL отримує EOF та завершується самостійно.
        engine.stdin.take();
        let status = engine.child.wait().expect("REPL завершився після EOF");
        engine.finish_readers();
        let mut stdout = String::new();
        let mut stderr = String::new();
        for line in engine.rx.try_iter() {
            match line.origin {
                Origin::Stdout => { stdout.push_str(&line.text); stdout.push('\n'); }
                Origin::Stderr => { stderr.push_str(&line.text); stderr.push('\n'); }
                Origin::System => panic!("не очікуємо вигаданий TUI-вивід"),
            }
        }
        assert!(status.success(), "помилка SENS REPL: {status}, stderr={stderr}");
        assert!(stdout.contains("укр"), "CLI не підтвердив повну українську поверхню: {stdout}");
        assert!(stdout.lines().any(|line| line.trim() == "так"), "SENS укр має повернути ратифіковане так: {stdout}, stderr={stderr}");
        assert!(stderr.is_empty(), "REPL stderr: {stderr}");
        println!("PINNED_SENS_UKR_REPL_OK output={}", stdout.trim());
    }

    #[test]
    #[ignore = "потрібен реальний SENS_PINNED_CLI; GitHub-hosted TUI CI запускає тест явно"]
    fn pinned_file_mode_smoke() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let expected = root.join("external/sens/target/release/my-lisp")
            .canonicalize().expect("збірка pinned sens-cli має існувати");
        let selected = PathBuf::from(std::env::var_os("SENS_PINNED_CLI")
            .expect("SENS_PINNED_CLI обов'язковий для реального smoke"));
        assert_eq!(
            selected.canonicalize().expect("шлях CLI має існувати"),
            expected,
            "заборонено перевіряти сторонній CLI або PATH fallback"
        );

        let workspace = std::env::temp_dir().join(format!(
            "my-idea-sens-file-smoke-{}-{}",
            std::process::id(), NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&workspace).expect("створити реальний workspace");
        let file = workspace.join("програма.lisp");
        // Десяткові числа у вихідному Lisp: результат інтерпретує тільки SENS.
        fs::write(&file, "(+ 2 3)\n(+ 4 5)\n").expect("записати багаторядковий Lisp");
        let (status, stdout, stderr) = run_file(&selected, &file);
        assert!(status.success(), "SENS не виконав програму: {status}, stderr={stderr}");
        assert_eq!(stdout.trim(), "9", "SENS має повернути значення останньої форми");
        assert!(stderr.is_empty(), "SENS помилково повідомив: {stderr}");

        fs::write(&file, "(\n").expect("записати помилкове джерело");
        let (status, stdout, stderr) = run_file(&selected, &file);
        assert!(!status.success(), "SENS прийняв синтаксичну помилку");
        assert!(stderr.contains("Parse error"), "немає реальної діагностики: {stderr}");
        println!(
            "PINNED_SENS_FILE_MODE_OK success_stdout={} failure_exit={} failure_stderr={}",
            "9", status, stderr.trim()
        );
        assert!(stdout.is_empty(), "некоректна програма не повинна мати результат");
        fs::remove_dir_all(workspace).expect("прибрати тимчасовий workspace");
    }
}
