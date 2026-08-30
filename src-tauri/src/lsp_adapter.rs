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
    position_request(
        "textDocument/definition",
        path,
        line,
        character,
        app,
        workspace,
        sessions,
    )
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
