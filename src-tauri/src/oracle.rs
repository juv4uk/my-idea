use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

/// my-lisp's default `--tcp` port (see my-lisp's `crates/my-lisp-cli/src/main.rs`).
const DEFAULT_PORT: u16 = 9999;
const TIMEOUT: Duration = Duration::from_secs(5);

/// Escapes a source string for embedding in a my-lisp string literal inside
/// the request envelope — backslash and `"` need escaping, and newlines are
/// flattened to spaces since the envelope is read one line at a time.
/// Екранує рядок джерела для вбудовування в my-lisp string literal у
/// request envelope.
fn escape_source(source: &str) -> String {
    source
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
}

/// Sends one `(request (id 1) (op ..) (source ..))` to my-lisp's TCP REPL
/// (`--tcp --protocol=sexpr`, see my-lisp's AGENTS.md) and returns the raw
/// `(response ...)` line — one connection per call, matching the oracle's
/// per-connection session isolation, so this is a one-shot query, not a
/// persistent link.
/// Надсилає один запит у TCP REPL my-lisp і повертає сиру відповідь.
pub fn query(op: &str, source: &str, port: Option<u16>) -> Result<String, String> {
    let port = port.unwrap_or(DEFAULT_PORT);
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .map_err(|e| format!("could not connect to my-lisp oracle on 127.0.0.1:{port}: {e}"))?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(TIMEOUT))
        .map_err(|e| e.to_string())?;

    let request = format!(
        "(request (id 1) (op {op}) (source \"{}\"))",
        escape_source(source)
    );
    writeln!(stream, "{request}").map_err(|e| format!("failed to send request: {e}"))?;

    let mut response = String::new();
    BufReader::new(stream)
        .read_line(&mut response)
        .map_err(|e| format!("failed to read response: {e}"))?;

    if response.trim().is_empty() {
        return Err("oracle closed the connection without responding".to_string());
    }
    Ok(response.trim().to_string())
}
