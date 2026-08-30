mod ecosystem;
mod oracle;
pub mod process_service;
mod swarm;
mod swarm_dashboard;

use my_lisp::Session;
use my_lisp_literate::SourceMode;
use serde::Serialize;
use std::{
    fs,
    fs::OpenOptions,
    io::Write,
    path::Component,
    path::{Path, PathBuf},
    sync::Mutex,
};
#[cfg(desktop)]
use tauri::AppHandle;
use tauri::State;
#[cfg(desktop)]
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct Workspace(Mutex<Option<PathBuf>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileNode {
    name: String,
    path: String,
    directory: bool,
    children: Vec<FileNode>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LispEvaluation {
    value: String,
    output: Vec<String>,
    ast: String,
    engine: &'static str,
}

/// Evaluates capability-free my-lisp code through the canonical Rust engine.
/// Uses a single-pass parse (`eval_parsed_expressions`) to avoid redundant parsing.
///
/// Обчислює код my-lisp без системних можливостей через канонічний Rust-рушій.
/// Використовує однопрохідний парсинг (`eval_parsed_expressions`), щоб уникнути повторного аналізу.
///
/// Wertet capability-freien my-lisp-Code mit der kanonischen Rust-Engine aus.
/// Verwendet Single-Pass-Parsing (`eval_parsed_expressions`), um doppeltes Parsing zu vermeiden.
#[tauri::command]
fn evaluate_my_lisp(source: String, mode: Option<String>) -> Result<LispEvaluation, String> {
    let mode_str = mode.as_deref().unwrap_or("my-lisp");
    let source_mode = if mode_str == "markdown" { SourceMode::Literate } else { SourceMode::PureLisp };
    let mut session = Session::default();
    let (result, forms) = my_lisp_literate::eval_literate(&source, source_mode, &mut session)
        .map_err(|error| error.to_string())?;
        
    Ok(LispEvaluation {
        value: result.value.to_string(),
        output: result.output,
        ast: format!("{forms:#?}"),
        engine: "my-lisp · Rust",
    })
}

/// Scans sibling repos (my-lisp, fpga-lisp, cml) and their machine-readable
/// contracts to report whether the ecosystem is currently coherent. System
/// Observatory MVP — see docs/system-observatory-vision.md.
///
/// Сканує сусідні репо (my-lisp, fpga-lisp, cml) та їхні машинно-читані
/// контракти, щоб показати, чи узгоджена зараз екосистема.
#[tauri::command]
fn ecosystem_status() -> ecosystem::EcosystemStatus {
    ecosystem::status()
}

/// Phase 1 of the Knowledge Graph tab (docs/knowledge-graph-design.md,
/// MYIDEA-KNOWLEDGE-GRAPH): repo-level nodes/edges derived from each
/// sibling's `repo.my` Swarm Contract v0.1 self-declaration, where present.
///
/// Фаза 1 вкладки Knowledge Graph: вузли/ребра рівня репо, виведені з
/// самодекларації `repo.my` кожного сусіда, де вона є.
#[tauri::command]
fn knowledge_graph() -> ecosystem::KnowledgeGraph {
    ecosystem::knowledge_graph()
}

/// Queries a running my-lisp `--tcp --protocol=sexpr` instance as a live
/// semantic oracle — `eval`/`diagnose`/`parse`/`contract-version` only, one
/// connection per call (the oracle isolates state per connection, so this
/// is a one-shot query, not a persistent session). Returns the raw
/// `(response ...)` s-expression; the frontend renders it as-is rather than
/// this backend re-interpreting my-lisp's own answer.
///
/// Запитує запущений my-lisp `--tcp --protocol=sexpr` як живий semantic
/// oracle. Повертає сиру `(response ...)` s-expression.
#[tauri::command]
fn oracle_query(source: String, op: Option<String>, port: Option<u16>) -> Result<oracle::OracleResponse, String> {
    oracle::query(op.as_deref().unwrap_or("eval"), &source, port)
}

/// Queries this app's own `swarm-node` (coordination-plane P2P mesh, see
/// my-lisp's `docs/swarm-mesh-v2.md` — separate from the `:9999` semantic
/// oracle) for its `(status)`: this node's identity/epoch/sync state, the
/// full peer presence list, `list-members`, and the task registry. Raw
/// s-expression, rendered as-is by the frontend — swarm-node's op set
/// (`status`/`list-members`/`presence`) is still evolving, so this
/// deliberately doesn't try to parse it into a fixed shape yet.
///
/// Запитує власний `swarm-node` цього застосунку на `(status)`.
#[tauri::command]
fn swarm_status(port: Option<u16>) -> Result<String, String> {
    swarm::query("(status)", port)
}

/// MYIDEA-SWARM-DASHBOARD: structured member/task view for the SWARM tab —
/// merges `(list-members)` (node id, presence, roles, capabilities) with
/// `(list-task-state)` (open/completed counts, current holder per task)
/// into one parsed `SwarmDashboard`, unlike `swarm_status` above which
/// stays raw-string on purpose.
///
/// Структурований вигляд членів/задач для вкладки SWARM.
#[tauri::command]
fn swarm_dashboard(port: Option<u16>) -> swarm_dashboard::SwarmDashboard {
    swarm_dashboard::dashboard(port)
}

#[cfg(test)]
mod language_adapter_tests {
    use super::evaluate_my_lisp;

    #[test]
    fn native_adapter_loads_bootstrap_library_and_preserves_exact_values() {
        let result = evaluate_my_lisp("(cons (second '(radio antenna)) (cons (/ 1 3) '()))".into(), Some("my-lisp".to_string()))
            .expect("native evaluation should succeed");
        assert_eq!(result.value, "(antenna 1/3)");
        assert_eq!(result.engine, "my-lisp · Rust");
    }
}

#[cfg(test)]
mod workspace_creation_tests {
    use super::create_new_workspace_file;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(label: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("my-idea-{label}-{nonce}"));
            fs::create_dir_all(&path).expect("test directory should be created");
            Self(
                path.canonicalize()
                    .expect("test directory should canonicalize"),
            )
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn creates_a_new_file_under_an_existing_workspace_parent() {
        let workspace = TestDirectory::new("create");
        fs::create_dir(workspace.0.join("src")).expect("source directory should be created");

        let created = create_new_workspace_file(&workspace.0, "src/main.wsm", "(quote ok)")
            .expect("new file should be created");

        assert_eq!(fs::read_to_string(created).unwrap(), "(quote ok)");
    }

    #[test]
    fn rejects_traversal_and_existing_files() {
        let workspace = TestDirectory::new("boundaries");
        assert!(create_new_workspace_file(&workspace.0, "../escape.wsm", "bad").is_err());

        fs::write(workspace.0.join("exists.wsm"), "original").unwrap();
        assert!(create_new_workspace_file(&workspace.0, "exists.wsm", "replacement").is_err());
        assert_eq!(
            fs::read_to_string(workspace.0.join("exists.wsm")).unwrap(),
            "original"
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_parent_symlink_that_leaves_the_workspace() {
        use std::os::unix::fs::symlink;

        let workspace = TestDirectory::new("symlink-workspace");
        let outside = TestDirectory::new("symlink-outside");
        symlink(&outside.0, workspace.0.join("outside-link")).unwrap();

        assert!(create_new_workspace_file(&workspace.0, "outside-link/escape.wsm", "bad").is_err());
        assert!(!outside.0.join("escape.wsm").exists());
    }
}

fn relative_text(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn scan(path: &Path, root: &Path) -> Result<Vec<FileNode>, String> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    entries.sort_by_key(|entry| {
        (
            !entry.path().is_dir(),
            entry.file_name().to_string_lossy().to_lowercase(),
        )
    });
    entries
        .into_iter()
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            !name.starts_with('.') && name != "node_modules" && name != "target" && name != "dist"
        })
        .map(|entry| {
            let path = entry.path();
            let directory = path.is_dir();
            Ok(FileNode {
                name: entry.file_name().to_string_lossy().into_owned(),
                path: relative_text(&path, root),
                directory,
                children: if directory {
                    scan(&path, root)?
                } else {
                    Vec::new()
                },
            })
        })
        .collect()
}

fn root(state: &State<'_, Workspace>) -> Result<PathBuf, String> {
    state
        .0
        .lock()
        .map_err(|_| "workspace lock is poisoned".to_string())?
        .clone()
        .ok_or_else(|| "no workspace is open".to_string())
}

fn safe_existing(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let candidate = root
        .join(relative)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if candidate.starts_with(root) {
        Ok(candidate)
    } else {
        Err("path escapes the workspace".into())
    }
}

fn create_new_workspace_file(
    root: &Path,
    relative: &str,
    contents: &str,
) -> Result<PathBuf, String> {
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("new file path must be a relative path without traversal".into());
    }

    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    let canonical_parent = root
        .join(parent)
        .canonicalize()
        .map_err(|error| format!("new file parent is not an existing directory: {error}"))?;
    if !canonical_parent.is_dir() || !canonical_parent.starts_with(root) {
        return Err("new file parent escapes the workspace".into());
    }

    let file_name = relative
        .file_name()
        .ok_or_else(|| "new file path has no file name".to_string())?;
    let destination = canonical_parent.join(file_name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)
        .map_err(|error| format!("new workspace file was not created: {error}"))?;
    file.write_all(contents.as_bytes())
        .map_err(|error| format!("new workspace file was not written: {error}"))?;
    Ok(destination)
}

#[tauri::command]
#[cfg(desktop)]
fn choose_workspace(app: AppHandle, state: State<'_, Workspace>) -> Result<Option<String>, String> {
    let Some(folder) = app
        .dialog()
        .file()
        .set_title("Open workspace")
        .blocking_pick_folder()
    else {
        return Ok(None);
    };
    let path = folder
        .into_path()
        .map_err(|error| error.to_string())?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    *state
        .0
        .lock()
        .map_err(|_| "workspace lock is poisoned".to_string())? = Some(path.clone());
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Reports the current mobile limitation without compiling desktop-only dialog APIs.
/// Повідомляє про поточне мобільне обмеження без компіляції desktop-only API діалогів.
/// Meldet die aktuelle Mobile-Einschränkung, ohne Desktop-Dialog-APIs zu kompilieren.
#[tauri::command]
#[cfg(mobile)]
fn choose_workspace(_state: State<'_, Workspace>) -> Result<Option<String>, String> {
    Err(concat!(
        "Opening workspace folders on Android requires Storage Access Framework support, which is planned. ",
        "Відкриття папок workspace на Android потребує підтримки Storage Access Framework, яку заплановано. ",
        "Das Öffnen von Workspace-Ordnern unter Android benötigt die geplante Unterstützung des Storage Access Framework."
    ).into())
}

#[tauri::command]
fn reopen_workspace(path: String, state: State<'_, Workspace>) -> Result<String, String> {
    let path = PathBuf::from(path)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !path.is_dir() {
        return Err("workspace is not a directory".into());
    }
    *state
        .0
        .lock()
        .map_err(|_| "workspace lock is poisoned".to_string())? = Some(path.clone());
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
fn list_workspace(state: State<'_, Workspace>) -> Result<Vec<FileNode>, String> {
    let root = root(&state)?;
    scan(&root, &root)
}

#[tauri::command]
fn read_workspace_file(path: String, state: State<'_, Workspace>) -> Result<String, String> {
    let root = root(&state)?;
    fs::read_to_string(safe_existing(&root, &path)?).map_err(|error| error.to_string())
}

#[tauri::command]
fn save_workspace_file(
    path: String,
    contents: String,
    state: State<'_, Workspace>,
) -> Result<(), String> {
    let root = root(&state)?;
    let path = safe_existing(&root, &path)?;
    if !path.is_file() {
        return Err("only existing workspace files can be saved".into());
    }
    fs::write(path, contents).map_err(|error| error.to_string())
}

#[tauri::command]
fn create_workspace_file(
    path: String,
    contents: String,
    state: State<'_, Workspace>,
) -> Result<(), String> {
    let root = root(&state)?;
    create_new_workspace_file(&root, &path, &contents).map(|_| ())
}

/// Opens a system «Save As» dialog and writes the file to the chosen location.
/// Відкриває системний діалог «Зберегти як» і записує файл у вибране місце.
/// Öffnet einen «Speichern unter»-Dialog und schreibt die Datei an den gewählten Speicherort.
#[tauri::command]
#[cfg(desktop)]
fn save_as_dialog(
    app: AppHandle,
    path: String,
    contents: String,
) -> Result<Option<String>, String> {
    let file_name = PathBuf::from(&path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "untitled".into());
    let chosen = app
        .dialog()
        .file()
        .set_title("Зберегти як · Save As · Speichern unter")
        .set_file_name(&file_name)
        .blocking_save_file();
    match chosen {
        Some(dest) => {
            let dest_path = dest.into_path().map_err(|e| e.to_string())?;
            fs::write(&dest_path, contents).map_err(|e| e.to_string())?;
            Ok(Some(dest_path.to_string_lossy().into_owned()))
        }
        None => Ok(None),
    }
}

/// Mobile stub – Save As dialog is not yet available on Android/iOS.
/// Мобільна заглушка – діалог «Зберегти як» ще не підтримується на Android/iOS.
/// Mobile-Stub – der «Speichern unter»-Dialog ist auf Android/iOS noch nicht verfügbar.
#[tauri::command]
#[cfg(mobile)]
fn save_as_dialog(
    _path: String,
    _contents: String,
) -> Result<Option<String>, String> {
    Err(concat!(
        "Save As dialog is not yet supported on mobile. ",
        "Діалог «Зберегти як» ще не підтримується на мобільних платформах. ",
        "Der «Speichern unter»-Dialog ist auf mobilen Plattformen noch nicht verfügbar."
    ).into())
}

/// Starts the native shell and exposes only workspace-scoped file operations.
/// Запускає оболонку й надає лише файлові операції в межах workspace.
/// Startet die Hülle mit ausschließlich arbeitsbereichsgebundenen Dateioperationen.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Workspace::default())
        .manage(process_service::ProcessService::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            choose_workspace,
            reopen_workspace,
            list_workspace,
            read_workspace_file,
            save_workspace_file,
            create_workspace_file,
            save_as_dialog,
            evaluate_my_lisp,
            ecosystem_status,
            knowledge_graph,
            oracle_query,
            swarm_status,
            swarm_dashboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running my-idea");
}
