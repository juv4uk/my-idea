use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::Duration,
};

const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
pub struct ServerProfile {
    pub name: String,
    pub executable: PathBuf,
    pub args: Vec<String>,
}

pub type NotificationSink = Arc<dyn Fn(Value) + Send + Sync + 'static>;

pub struct LanguageServer {
    profile: String,
    next_id: AtomicU64,
    stdin: Arc<Mutex<ChildStdin>>,
    child: Mutex<Child>,
    pending: Arc<Mutex<HashMap<u64, mpsc::Sender<Value>>>>,
}

impl LanguageServer {
    pub fn spawn(
        workspace: &Path,
        profile: ServerProfile,
        notifications: NotificationSink,
    ) -> Result<Self, String> {
        let workspace = workspace
            .canonicalize()
            .map_err(|error| format!("LSP workspace is unavailable: {error}"))?;
        if !workspace.is_dir() {
            return Err("LSP workspace is not a directory".into());
        }

        let mut child = Command::new(&profile.executable)
            .args(&profile.args)
            .current_dir(workspace)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("could not start LSP server '{}': {error}", profile.name))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "LSP server stdin was not captured".to_string())?;
        let stdin = Arc::new(Mutex::new(stdin));
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "LSP server stdout was not captured".to_string())?;
        let pending: Arc<Mutex<HashMap<u64, mpsc::Sender<Value>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let reader_pending = pending.clone();
        let reader_notifications = notifications.clone();
        let reader_stdin = stdin.clone();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                match read_message(&mut reader) {
                    Ok(Some(message)) => {
                        let method = message.get("method").and_then(Value::as_str);
                        let response_id = message.get("id").and_then(Value::as_u64);
                        if let Some(method) = method {
                            reader_notifications(message.clone());
                            if let Some(id) = response_id {
                                let response = server_request_response(id, method, &message);
                                if let Ok(mut writer) = reader_stdin.lock() {
                                    let _ = write_message(&mut *writer, &response);
                                }
                            }
                        } else if let Some(id) = response_id {
                            if let Some(sender) = reader_pending
                                .lock()
                                .ok()
                                .and_then(|mut requests| requests.remove(&id))
                            {
                                let _ = sender.send(message);
                            }
                        }
                    }
                    Ok(None) => break,
                    Err(error) => {
                        reader_notifications(json!({
                            "jsonrpc": "2.0",
                            "method": "$/myIdeaTransportError",
                            "params": {"message": error}
                        }));
                        break;
                    }
                }
            }
        });

        // LSP stderr is diagnostic logging, never protocol traffic. Drain it
        // so a verbose server cannot block on a full pipe.
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(
                move || {
                    for _ in BufReader::new(stderr).lines().map_while(Result::ok) {}
                },
            );
        }

        Ok(Self {
            profile: profile.name,
            next_id: AtomicU64::new(1),
            stdin,
            child: Mutex::new(child),
            pending,
        })
    }

    pub fn request(&self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = mpsc::channel();
        self.pending
            .lock()
            .map_err(|_| "LSP pending-request lock is poisoned".to_string())?
            .insert(id, sender);
        let message = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        if let Err(error) = self.write(&message) {
            if let Ok(mut pending) = self.pending.lock() {
                pending.remove(&id);
            }
            return Err(error);
        }
        receiver.recv_timeout(RESPONSE_TIMEOUT).map_err(|_| {
            if let Ok(mut pending) = self.pending.lock() {
                pending.remove(&id);
            }
            format!("LSP server '{}' timed out on {method}", self.profile)
        })
    }

    pub fn notify(&self, method: &str, params: Value) -> Result<(), String> {
        self.write(&json!({"jsonrpc": "2.0", "method": method, "params": params}))
    }

    pub fn shutdown(&self) -> Result<(), String> {
        if self
            .child
            .lock()
            .map_err(|_| "LSP child lock is poisoned".to_string())?
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Ok(());
        }
        let _ = self.request("shutdown", Value::Null);
        let _ = self.notify("exit", Value::Null);
        let mut child = self
            .child
            .lock()
            .map_err(|_| "LSP child lock is poisoned".to_string())?;
        for _ in 0..20 {
            if child
                .try_wait()
                .map_err(|error| error.to_string())?
                .is_some()
            {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(25));
        }
        child.kill().map_err(|error| error.to_string())?;
        child.wait().map_err(|error| error.to_string())?;
        Ok(())
    }

    fn write(&self, message: &Value) -> Result<(), String> {
        let mut stdin = self
            .stdin
            .lock()
            .map_err(|_| "LSP stdin lock is poisoned".to_string())?;
        write_message(&mut *stdin, message)
    }
}

fn server_request_response(id: u64, method: &str, request: &Value) -> Value {
    let result = match method {
        "workspace/configuration" => {
            let count = request["params"]["items"].as_array().map_or(0, Vec::len);
            Value::Array(vec![Value::Null; count])
        }
        "workspace/workspaceFolders" => Value::Null,
        "client/registerCapability"
        | "client/unregisterCapability"
        | "window/workDoneProgress/create" => Value::Null,
        _ => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("unsupported server request: {method}")}
            });
        }
    };
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

impl Drop for LanguageServer {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn write_message(writer: &mut impl Write, message: &Value) -> Result<(), String> {
    let body = serde_json::to_vec(message).map_err(|error| error.to_string())?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len()).map_err(|error| error.to_string())?;
    writer.write_all(&body).map_err(|error| error.to_string())?;
    writer.flush().map_err(|error| error.to_string())
}

fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
    let mut content_length = None;
    loop {
        let mut header = String::new();
        let read = reader
            .read_line(&mut header)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Ok(None);
        }
        if header == "\r\n" || header == "\n" {
            break;
        }
        if let Some(value) = header.strip_prefix("Content-Length:").map(str::trim) {
            content_length = Some(
                value
                    .parse::<usize>()
                    .map_err(|_| "invalid LSP Content-Length header".to_string())?,
            );
        }
    }
    let length = content_length.ok_or_else(|| "LSP message has no Content-Length".to_string())?;
    let mut body = vec![0; length];
    reader
        .read_exact(&mut body)
        .map_err(|error| error.to_string())?;
    serde_json::from_slice(&body).map(Some).map_err(|error| {
        format!(
            "invalid LSP JSON: {error}; body={}",
            String::from_utf8_lossy(&body)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn content_length_round_trip_preserves_json_rpc_message() {
        let message = json!({"jsonrpc": "2.0", "id": 7, "method": "initialize", "params": {}});
        let mut bytes = Vec::new();
        write_message(&mut bytes, &message).unwrap();
        let decoded = read_message(&mut Cursor::new(bytes)).unwrap().unwrap();
        assert_eq!(decoded, message);
    }

    #[test]
    fn framing_rejects_missing_or_invalid_content_length() {
        assert!(read_message(&mut Cursor::new(b"\r\n{}".to_vec())).is_err());
        assert!(
            read_message(&mut Cursor::new(b"Content-Length: nope\r\n\r\n{}".to_vec())).is_err()
        );
    }

    #[test]
    fn answers_language_server_configuration_requests() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "workspace/configuration",
            "params": {"items": [{"section": "rust-analyzer"}, {"section": "rust-analyzer.cargo"}]}
        });
        assert_eq!(
            server_request_response(9, "workspace/configuration", &request),
            json!({"jsonrpc": "2.0", "id": 9, "result": [null, null]})
        );
    }

    #[test]
    #[ignore = "requires MY_IDEA_LSP_TEST_BIN and a real language server"]
    fn initializes_a_real_configured_language_server() {
        let executable = std::env::var("MY_IDEA_LSP_TEST_BIN").unwrap();
        let args: Vec<String> = serde_json::from_str(
            &std::env::var("MY_IDEA_LSP_TEST_ARGS_JSON").unwrap_or_else(|_| "[]".into()),
        )
        .unwrap();
        let workspace = std::env::current_dir().unwrap();
        let server = LanguageServer::spawn(
            &workspace,
            ServerProfile {
                name: "live-test".into(),
                executable: executable.into(),
                args,
            },
            Arc::new(|_| {}),
        )
        .unwrap();
        let response = server
            .request(
                "initialize",
                json!({
                    "processId": null,
                    "rootUri": format!("file://{}", workspace.display()),
                    "capabilities": {}
                }),
            )
            .unwrap();
        assert!(response.get("result").is_some(), "{response}");
        server.notify("initialized", json!({})).unwrap();
        server.shutdown().unwrap();
    }

    #[test]
    #[ignore = "requires MY_IDEA_LSP_TEST_BIN=rust-analyzer"]
    fn real_rust_analyzer_serves_document_symbols() {
        let executable = std::env::var("MY_IDEA_LSP_TEST_BIN").unwrap();
        let workspace = std::env::current_dir().unwrap();
        let server = LanguageServer::spawn(
            &workspace,
            ServerProfile {
                name: "rust-analyzer-live-test".into(),
                executable: executable.into(),
                args: Vec::new(),
            },
            Arc::new(|_| {}),
        )
        .unwrap();
        server
            .request(
                "initialize",
                json!({
                    "processId": null,
                    "rootUri": format!("file://{}", workspace.display()),
                    "capabilities": {"workspace": {"configuration": true}}
                }),
            )
            .unwrap();
        server.notify("initialized", json!({})).unwrap();
        let uri = "file:///tmp/my-idea-rust-analyzer-live.rs";
        server
            .notify(
                "textDocument/didOpen",
                json!({"textDocument": {
                    "uri": uri,
                    "languageId": "rust",
                    "version": 1,
                    "text": "fn viveka_lsp_witness() -> u32 { 42 }"
                }}),
            )
            .unwrap();
        let symbols = server
            .request(
                "textDocument/documentSymbol",
                json!({"textDocument": {"uri": uri}}),
            )
            .unwrap();
        assert!(
            symbols.to_string().contains("viveka_lsp_witness"),
            "{symbols}"
        );
        server.shutdown().unwrap();
    }

    #[test]
    #[ignore = "requires MY_IDEA_LSP_TEST_BIN=my-lisp and its lsp argument"]
    fn real_wsm_server_publishes_diagnostics_and_completion() {
        let executable = std::env::var("MY_IDEA_LSP_TEST_BIN").unwrap();
        let workspace = std::env::current_dir().unwrap();
        let (sender, receiver) = mpsc::channel();
        let server = LanguageServer::spawn(
            &workspace,
            ServerProfile {
                name: "WsmLS-live-test".into(),
                executable: executable.into(),
                args: vec!["lsp".into()],
            },
            Arc::new(move |message| sender.send(message).unwrap()),
        )
        .unwrap();
        server
            .request(
                "initialize",
                json!({"processId": null, "rootUri": null, "capabilities": {}}),
            )
            .unwrap();
        server.notify("initialized", json!({})).unwrap();
        let uri = "file:///tmp/my-idea-live-test.wsm";
        server
            .notify(
                "textDocument/didOpen",
                json!({"textDocument": {
                    "uri": uri, "languageId": "wsm", "version": 1, "text": "(cons"
                }}),
            )
            .unwrap();
        let diagnostics = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(
            diagnostics["method"], "textDocument/publishDiagnostics",
            "{diagnostics}"
        );
        assert_ne!(diagnostics["params"]["diagnostics"], json!([]));

        server
            .notify(
                "textDocument/didChange",
                json!({
                    "textDocument": {"uri": uri, "version": 2},
                    "contentChanges": [{"text": "(cons (quote a) (quote ()))"}]
                }),
            )
            .unwrap();
        let _ = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        let completion = server
            .request(
                "textDocument/completion",
                json!({
                    "textDocument": {"uri": uri},
                    "position": {"line": 0, "character": 2}
                }),
            )
            .unwrap();
        assert!(completion.to_string().contains("cons"), "{completion}");
        server.shutdown().unwrap();
    }
}
