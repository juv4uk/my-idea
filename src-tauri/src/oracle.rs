use my_lisp::{parse, Expr, ExprKind};
use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

/// my-lisp's default `--tcp` port (see my-lisp's `crates/my-lisp-cli/src/main.rs`).
const DEFAULT_PORT: u16 = 9999;
const TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_ATTEMPTS: u32 = 3;
const CONNECT_RETRY_DELAY: Duration = Duration::from_millis(300);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OracleResponse {
    /// "ok" or "error", read straight from the response's `(status ..)` field.
    pub status: String,
    /// Error kind, e.g. `type-error` — only set when `status` is `error`.
    pub kind: Option<String>,
    /// Human-readable error message — only set when `status` is `error`.
    pub message: Option<String>,
    /// The full `(response ...)` line, for the cases the structured fields
    /// above don't cover (e.g. rendering `(value ...)` in my-lisp syntax).
    pub raw: String,
}

fn as_list(expr: &Expr) -> Option<&[Expr]> {
    match &expr.kind {
        ExprKind::List(items) => Some(items),
        _ => None,
    }
}

fn symbol_name(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Symbol(name) => Some(name),
        _ => None,
    }
}

fn string_value(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::String(value) => Some(value.to_string()),
        _ => None,
    }
}

fn field<'a>(items: &'a [Expr], key: &str) -> Option<&'a Expr> {
    items.iter().find_map(|item| {
        let pair = as_list(item)?;
        (pair.first().and_then(symbol_name)? == key).then(|| pair.get(1)).flatten()
    })
}

/// Parses a `(response (id ..) (status ..) ...)` line into its `status`,
/// and — for errors — `kind`/`message`. Falls back to `status: "unknown"`
/// with the raw line intact if the response doesn't parse as expected; the
/// oracle's own reply is trusted for correctness, not re-validated here.
/// Парсить `(response ...)` у status/kind/message; сирий рядок лишається
/// доступним, якщо структуру не вдалось розпізнати.
fn parse_response(raw: &str) -> OracleResponse {
    let parsed = parse(raw).ok().and_then(|forms| forms.into_iter().next());
    let fields = parsed.as_ref().and_then(as_list);

    let status = fields
        .and_then(|f| field(f, "status"))
        .and_then(symbol_name)
        .unwrap_or("unknown")
        .to_string();
    let kind = fields.and_then(|f| field(f, "kind")).and_then(symbol_name).map(str::to_string);
    let message = fields.and_then(|f| field(f, "message")).and_then(string_value);

    OracleResponse { status, kind, message, raw: raw.to_string() }
}

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

fn connect(port: u16) -> Result<TcpStream, String> {
    let mut last_err = String::new();
    for attempt in 0..CONNECT_ATTEMPTS {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => return Ok(stream),
            Err(e) => last_err = e.to_string(),
        }
        if attempt + 1 < CONNECT_ATTEMPTS {
            std::thread::sleep(CONNECT_RETRY_DELAY);
        }
    }
    Err(format!(
        "could not connect to my-lisp oracle on 127.0.0.1:{port} after {CONNECT_ATTEMPTS} attempts: {last_err}"
    ))
}

/// Sends one `(request (id 1) (op ..) (source ..))` to my-lisp's TCP REPL
/// (`--tcp --protocol=sexpr`, see my-lisp's AGENTS.md) and returns the
/// parsed `(response ...)` — one connection per call, matching the oracle's
/// per-connection session isolation, so this is a one-shot query, not a
/// persistent link. Retries the initial connection a few times (the oracle
/// may just be starting up), but not the request/response round-trip.
/// Надсилає один запит у TCP REPL my-lisp і повертає розпарсену відповідь.
pub fn query(op: &str, source: &str, port: Option<u16>) -> Result<OracleResponse, String> {
    let port = port.unwrap_or(DEFAULT_PORT);
    let mut stream = connect(port)?;
    stream.set_read_timeout(Some(TIMEOUT)).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(TIMEOUT)).map_err(|e| e.to_string())?;

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
    Ok(parse_response(response.trim()))
}
