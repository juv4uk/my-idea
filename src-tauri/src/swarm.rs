use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

/// This app's own `swarm-node` (see my-lisp's `docs/swarm-mesh-v2.md`) —
/// the P2P coordination-plane node my-idea runs alongside the `:9999`
/// semantic oracle. Not configurable yet; every agent's swarm-node port
/// is a manual `--port` choice at spawn time.
const DEFAULT_PORT: u16 = 9104;
const TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_ATTEMPTS: u32 = 3;
const CONNECT_RETRY_DELAY: Duration = Duration::from_millis(300);

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
        "could not connect to swarm-node on 127.0.0.1:{port} after {CONNECT_ATTEMPTS} attempts: {last_err}"
    ))
}

/// Sends one op (e.g. `"(status)"`, `"(list-members)"`) to this agent's own
/// `swarm-node` and returns the raw response line. Unlike the `:9999`
/// oracle, swarm-node keeps the connection open after answering (it's
/// built for a persistent client, not one-shot calls), so this reads
/// exactly one line rather than waiting for EOF.
///
/// Надсилає один op до власного `swarm-node` цього агента, повертає сиру
/// відповідь. На відміну від `:9999`, з'єднання лишається відкритим —
/// читаємо рівно один рядок, не чекаючи EOF.
pub fn query(op: &str, port: Option<u16>) -> Result<String, String> {
    let port = port.unwrap_or(DEFAULT_PORT);
    let mut stream = connect(port)?;
    stream.set_read_timeout(Some(TIMEOUT)).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(TIMEOUT)).map_err(|e| e.to_string())?;

    writeln!(stream, "{op}").map_err(|e| format!("failed to send request: {e}"))?;

    let mut response = String::new();
    BufReader::new(stream)
        .read_line(&mut response)
        .map_err(|e| format!("failed to read response: {e}"))?;

    if response.trim().is_empty() {
        return Err("swarm-node closed the connection without responding".to_string());
    }
    Ok(response.trim().to_string())
}
