//! Реальне виконання через той самий Engine, що використовується в Ratatui F5.
//! Немає фальшивого evaluator: код виконує зібраний CLI з pinned external/sens.
#[path = "../src/engine.rs"]
mod engine;

use std::fs;
use std::path::Path;
use std::process::ExitStatus;
use std::time::{Duration, Instant};

fn execute(binary: &Path, source: &Path) -> (ExitStatus, Vec<engine::Line>) {
    let mut child = engine::Engine::spawn_file(binary, source)
        .expect("запуск реального pinned SENS CLI");
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().expect("стан CLI") { break status; }
        if Instant::now() > deadline {
            child.stop();
            panic!("реальний SENS CLI не завершився за 60 секунд");
        }
        std::thread::sleep(Duration::from_millis(30));
    };
    // Дочитуємо реальні stdout/stderr ПЕРЕД тим, як публікувати exit-status.
    child.finish_readers();
    let lines = std::iter::from_fn(|| child.rx.try_recv().ok()).collect();
    (status, lines)
}

fn stream(lines: &[engine::Line], origin: engine::Origin) -> String {
    lines.iter().filter(|line| line.origin == origin)
        .map(|line| line.text.as_str()).collect::<Vec<_>>().join("\n")
}

#[test]
#[ignore = "потрібен зібраний pinned SENS CLI; GitHub-hosted TUI workflow запускає явно"]
fn pinned_file_mode_smoke() {
    let resolved = engine::resolve_bin().expect(
        "саме pinned external/sens має бути ініціалізований і зібраний, не PATH"
    );
    eprintln!("REAL_SENS_SHA={} CLI={}", resolved.pinned_sha, resolved.path.display());

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir()
        .join(format!("my-idea-real-sens-{}-{stamp}", std::process::id()));
    fs::create_dir(&root).expect("каталог для реального Lisp файлу");
    let source = root.join("програма.lisp");

    // Кілька повних форм: file-mode CLI, не stdin-рядки readline REPL.
    fs::write(&source, "(+ 2 3)\n(+ 4 5)\n").unwrap();
    let (status, lines) = execute(&resolved.path, &source);
    let stdout = stream(&lines, engine::Origin::Stdout);
    let stderr = stream(&lines, engine::Origin::Stderr);
    assert!(status.success(), "SENS exit {status}: stdout={stdout}; stderr={stderr}");
    assert_eq!(stdout.trim(), "9", "останнє значення всього багаторядкового файла");
    assert!(stderr.trim().is_empty(), "неочікуваний stderr: {stderr}");

    // Справжній parser має дати nonzero exit і точну помилку.
    fs::write(&source, "(+ 1\n").unwrap();
    let (status, lines) = execute(&resolved.path, &source);
    let stderr = stream(&lines, engine::Origin::Stderr);
    assert!(!status.success(), "незакрита форма не повинна проходити: {status}");
    assert!(stderr.contains("Parse error"), "реальна синтаксична помилка: {stderr}");

    fs::remove_dir_all(&root).unwrap();
}
