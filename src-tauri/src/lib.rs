use my_lisp::{eval_program, parse, Session};
use serde::Serialize;
use std::{
    fs,
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
/// Обчислює код my-lisp без системних можливостей через канонічний Rust-рушій.
/// Wertet capability-freien my-lisp-Code mit der kanonischen Rust-Engine aus.
#[tauri::command]
fn evaluate_my_lisp(source: String) -> Result<LispEvaluation, String> {
    let forms = parse(&source).map_err(|error| error.to_string())?;
    let mut session = Session::default();
    eval_program(include_str!("../../lib/core.my"), &mut session)
        .map_err(|error| error.to_string())?;
    let result = eval_program(&source, &mut session).map_err(|error| error.to_string())?;
    Ok(LispEvaluation {
        value: result.value.to_string(),
        output: result.output,
        ast: format!("{forms:#?}"),
        engine: "my-lisp · Rust",
    })
}

#[cfg(test)]
mod language_adapter_tests {
    use super::evaluate_my_lisp;

    #[test]
    fn native_adapter_loads_bootstrap_library_and_preserves_exact_values() {
        let result = evaluate_my_lisp("(cons (second '(radio antenna)) (cons (/ 1 3) '()))".into())
            .expect("native evaluation should succeed");
        assert_eq!(result.value, "(antenna 1/3)");
        assert_eq!(result.engine, "my-lisp · Rust");
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

/// Starts the native shell and exposes only workspace-scoped file operations.
/// Запускає оболонку й надає лише файлові операції в межах workspace.
/// Startet die Hülle mit ausschließlich arbeitsbereichsgebundenen Dateioperationen.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Workspace::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            choose_workspace,
            reopen_workspace,
            list_workspace,
            read_workspace_file,
            save_workspace_file,
            evaluate_my_lisp
        ])
        .run(tauri::generate_context!())
        .expect("error while running my-idea");
}
