use crate::{
    lsp_client::{LanguageServer, ServerProfile},
    root, safe_existing, Workspace,
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, State, Url};

#[derive(Default)]
pub struct LspSessions(Mutex<HashMap<String, Arc<LanguageServer>>>);

impl LspSessions {
    fn wsm(&self, workspace: &Path, app: &AppHandle) -> Result<Arc<LanguageServer>, String> {
        let mut sessions = self
            .0
            .lock()
            .map_err(|_| "LSP session lock is poisoned".to_string())?;
        let session_key = format!("wsm:{}", workspace.display());
        if let Some(server) = sessions.get(&session_key) {
            return Ok(server.clone());
        }
        let app_handle = app.clone();
        let server = Arc::new(LanguageServer::spawn(
            workspace,
            ServerProfile {
                name: "WsmLS".into(),
                executable: find_my_lisp(workspace),
                args: vec!["lsp".into()],
            },
            Arc::new(move |message| {
                let _ = app_handle.emit("lsp-message", message);
            }),
        )?);
        let initialize = server.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": directory_uri(workspace)?,
                "capabilities": {
                    "textDocument": {
                        "publishDiagnostics": {"relatedInformation": true},
                        "completion": {"completionItem": {"snippetSupport": false}},
                        "hover": {"contentFormat": ["markdown", "plaintext"]},
                        "documentSymbol": {},
                        "definition": {"linkSupport": true}
                    }
                },
                "clientInfo": {"name": "my-idea", "version": env!("CARGO_PKG_VERSION")}
            }),
        )?;
        if initialize.get("error").is_some() {
            return Err(format!("WsmLS initialize failed: {initialize}"));
        }
        server.notify("initialized", json!({}))?;
        sessions.insert(session_key, server.clone());
        Ok(server)
    }

    fn rust(&self, workspace: &Path, app: &AppHandle) -> Result<Arc<LanguageServer>, String> {
        let mut sessions = self
            .0
            .lock()
            .map_err(|_| "LSP session lock is poisoned".to_string())?;
        let session_key = format!("rust:{}", workspace.display());
        if let Some(server) = sessions.get(&session_key) {
            return Ok(server.clone());
        }
        let app_handle = app.clone();
        let server = Arc::new(LanguageServer::spawn(
            workspace,
            ServerProfile {
                name: "rust-analyzer".into(),
                executable: std::env::var_os("MY_IDEA_RUST_ANALYZER_BIN")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| "rust-analyzer".into()),
                args: Vec::new(),
            },
            Arc::new(move |message| {
                let _ = app_handle.emit("lsp-message", message);
            }),
        )?);
        let initialize = server.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": directory_uri(workspace)?,
                "workspaceFolders": [{"uri": directory_uri(workspace)?, "name": "workspace"}],
                "capabilities": {
                    "workspace": {"configuration": true, "workspaceFolders": true},
                    "textDocument": {
                        "publishDiagnostics": {"relatedInformation": true},
                        "completion": {"completionItem": {"snippetSupport": false}},
                        "hover": {"contentFormat": ["markdown", "plaintext"]},
                        "documentSymbol": {},
                        "definition": {"linkSupport": true}
                    },
                    "window": {"workDoneProgress": true}
                },
                "clientInfo": {"name": "my-idea", "version": env!("CARGO_PKG_VERSION")}
            }),
        )?;
        if initialize.get("error").is_some() {
            return Err(format!("rust-analyzer initialize failed: {initialize}"));
        }
        server.notify("initialized", json!({}))?;
        sessions.insert(session_key, server.clone());
        Ok(server)
    }
}

fn find_my_lisp(workspace: &Path) -> PathBuf {
    if let Some(path) = std::env::var_os("MY_IDEA_MY_LISP_BIN") {
        return path.into();
    }
    let sibling = workspace
        .parent()
        .map(|parent| parent.join("my-lisp/target/release/my-lisp"));
    if let Some(path) = sibling.filter(|path| path.is_file()) {
        return path;
    }
    "my-lisp".into()
}

fn directory_uri(path: &Path) -> Result<String, String> {
    Url::from_directory_path(path)
        .map(|uri| uri.to_string())
        .map_err(|_| "workspace path cannot be represented as a file URI".into())
}

fn document_uri(root: &Path, relative: &str) -> Result<String, String> {
    let path = safe_existing(root, relative)?;
    Url::from_file_path(path)
        .map(|uri| uri.to_string())
        .map_err(|_| "document path cannot be represented as a file URI".into())
}

fn rust_document(root: &Path, relative: &str) -> Result<(PathBuf, String), String> {
    let document = safe_existing(root, relative)?;
    let uri = Url::from_file_path(&document)
        .map(|uri| uri.to_string())
        .map_err(|_| "document path cannot be represented as a file URI".to_string())?;
    let mut directory = document
        .parent()
        .ok_or_else(|| "Rust document has no parent directory".to_string())?;
    loop {
        if directory.join("Cargo.toml").is_file() {
            return Ok((directory.to_path_buf(), uri));
        }
        if directory == root {
            break;
        }
        let Some(parent) = directory.parent() else {
            break;
        };
        if !parent.starts_with(root) {
            break;
        }
        directory = parent;
    }
    Ok((root.to_path_buf(), uri))
}

fn definition_target(root: &Path, response: Value) -> Result<Value, String> {
    let result = &response["result"];
    let location = if let Some(locations) = result.as_array() {
        locations.first().unwrap_or(&Value::Null)
    } else {
        result
    };
    if location.is_null() {
        return Ok(json!({"result": null}));
    }
    let uri = location
        .get("targetUri")
        .or_else(|| location.get("uri"))
        .and_then(Value::as_str)
        .ok_or_else(|| "definition response has no file URI".to_string())?;
    let range = location
        .get("targetSelectionRange")
        .or_else(|| location.get("range"))
        .ok_or_else(|| "definition response has no range".to_string())?;
    let absolute = Url::parse(uri)
        .map_err(|error| format!("invalid definition URI: {error}"))?
        .to_file_path()
        .map_err(|_| "definition URI is not a local file".to_string())?;
    let relative = absolute
        .strip_prefix(root)
        .map_err(|_| "language server returned a definition outside the workspace".to_string())?;
    Ok(json!({"result": {
        "path": relative.to_string_lossy().replace('\\', "/"),
        "line": range["start"]["line"],
        "character": range["start"]["character"]
    }}))
}

#[tauri::command]
pub fn wsm_lsp_open(
    path: String,
    text: String,
    version: i32,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<(), String> {
    let root = root(&workspace)?;
    sessions.wsm(&root, &app)?.notify(
        "textDocument/didOpen",
        json!({"textDocument": {
            "uri": document_uri(&root, &path)?,
            "languageId": "wsm",
            "version": version,
            "text": text
        }}),
    )
}

#[tauri::command]
pub fn wsm_lsp_change(
    path: String,
    text: String,
    version: i32,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<(), String> {
    let root = root(&workspace)?;
    sessions.wsm(&root, &app)?.notify(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": document_uri(&root, &path)?, "version": version},
            "contentChanges": [{"text": text}]
        }),
    )
}

#[tauri::command]
pub fn wsm_lsp_close(
    path: String,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<(), String> {
    let root = root(&workspace)?;
    sessions.wsm(&root, &app)?.notify(
        "textDocument/didClose",
        json!({"textDocument": {"uri": document_uri(&root, &path)?}}),
    )
}

fn position_request(
    method: &str,
    path: String,
    line: u32,
    character: u32,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<Value, String> {
    let root = root(&workspace)?;
    sessions.wsm(&root, &app)?.request(
        method,
        json!({
            "textDocument": {"uri": document_uri(&root, &path)?},
            "position": {"line": line, "character": character}
        }),
    )
}

#[tauri::command]
pub fn wsm_lsp_completion(
    path: String,
    line: u32,
    character: u32,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<Value, String> {
    position_request(
        "textDocument/completion",
        path,
        line,
        character,
        app,
        workspace,
        sessions,
    )
}

#[tauri::command]
pub fn wsm_lsp_hover(
    path: String,
    line: u32,
    character: u32,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<Value, String> {
    position_request(
        "textDocument/hover",
        path,
        line,
        character,
        app,
        workspace,
        sessions,
    )
}

#[tauri::command]
pub fn wsm_lsp_definition(
    path: String,
    line: u32,
    character: u32,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<Value, String> {
    let root = root(&workspace)?;
    let response = sessions.wsm(&root, &app)?.request(
        "textDocument/definition",
        json!({
            "textDocument": {"uri": document_uri(&root, &path)?},
            "position": {"line": line, "character": character}
        }),
    )?;
    definition_target(&root, response)
}

#[tauri::command]
pub fn wsm_lsp_symbols(
    path: String,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<Value, String> {
    let root = root(&workspace)?;
    sessions.wsm(&root, &app)?.request(
        "textDocument/documentSymbol",
        json!({"textDocument": {"uri": document_uri(&root, &path)?}}),
    )
}

#[tauri::command]
pub fn rust_lsp_open(
    path: String,
    text: String,
    version: i32,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<(), String> {
    let root = root(&workspace)?;
    let (project, uri) = rust_document(&root, &path)?;
    sessions.rust(&project, &app)?.notify(
        "textDocument/didOpen",
        json!({"textDocument": {
            "uri": uri,
            "languageId": "rust",
            "version": version,
            "text": text
        }}),
    )
}

#[tauri::command]
pub fn rust_lsp_change(
    path: String,
    text: String,
    version: i32,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<(), String> {
    let root = root(&workspace)?;
    let (project, uri) = rust_document(&root, &path)?;
    sessions.rust(&project, &app)?.notify(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": uri, "version": version},
            "contentChanges": [{"text": text}]
        }),
    )
}

#[tauri::command]
pub fn rust_lsp_close(
    path: String,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<(), String> {
    let root = root(&workspace)?;
    let (project, uri) = rust_document(&root, &path)?;
    sessions.rust(&project, &app)?.notify(
        "textDocument/didClose",
        json!({"textDocument": {"uri": uri}}),
    )
}

fn rust_position_request(
    method: &str,
    path: String,
    line: u32,
    character: u32,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<Value, String> {
    let root = root(&workspace)?;
    let (project, uri) = rust_document(&root, &path)?;
    sessions.rust(&project, &app)?.request(
        method,
        json!({
            "textDocument": {"uri": uri},
            "position": {"line": line, "character": character}
        }),
    )
}

macro_rules! rust_position_command {
    ($name:ident, $method:literal) => {
        #[tauri::command]
        pub fn $name(
            path: String,
            line: u32,
            character: u32,
            app: AppHandle,
            workspace: State<'_, Workspace>,
            sessions: State<'_, LspSessions>,
        ) -> Result<Value, String> {
            rust_position_request($method, path, line, character, app, workspace, sessions)
        }
    };
}

rust_position_command!(rust_lsp_completion, "textDocument/completion");
rust_position_command!(rust_lsp_hover, "textDocument/hover");

#[tauri::command]
pub fn rust_lsp_definition(
    path: String,
    line: u32,
    character: u32,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<Value, String> {
    let root = root(&workspace)?;
    let (project, uri) = rust_document(&root, &path)?;
    let response = sessions.rust(&project, &app)?.request(
        "textDocument/definition",
        json!({
            "textDocument": {"uri": uri},
            "position": {"line": line, "character": character}
        }),
    )?;
    definition_target(&root, response)
}

#[tauri::command]
pub fn rust_lsp_symbols(
    path: String,
    app: AppHandle,
    workspace: State<'_, Workspace>,
    sessions: State<'_, LspSessions>,
) -> Result<Value, String> {
    let root = root(&workspace)?;
    let (project, uri) = rust_document(&root, &path)?;
    sessions.rust(&project, &app)?.request(
        "textDocument/documentSymbol",
        json!({"textDocument": {"uri": uri}}),
    )
}

#[cfg(test)]
mod tests {
    use super::{definition_target, rust_document};
    use serde_json::json;
    use std::path::Path;
    use tauri::Url;

    #[test]
    fn rust_document_uses_nearest_cargo_root() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("src-tauri has a repository parent");
        let (project, uri) = rust_document(workspace, "src-tauri/src/lsp_adapter.rs").unwrap();
        assert_eq!(project, workspace.join("src-tauri"));
        assert!(uri.ends_with("/src-tauri/src/lsp_adapter.rs"));
    }

    #[test]
    fn definition_target_is_workspace_relative() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("src-tauri has a repository parent");
        let target = workspace.join("src-tauri/src/lsp_adapter.rs");
        let response = json!({"result": [{
            "targetUri": Url::from_file_path(target).unwrap().to_string(),
            "targetSelectionRange": {"start": {"line": 7, "character": 3}, "end": {"line": 7, "character": 9}}
        }]});
        assert_eq!(
            definition_target(workspace, response).unwrap(),
            json!({"result": {"path": "src-tauri/src/lsp_adapter.rs", "line": 7, "character": 3}})
        );
    }
}
