//! Shared mock OpenAI-compatible server and CLI harness for the
//! end-to-end tests. No real network access occurs: the server
//! listens on loopback and serves canned chat completions.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

use tempfile::TempDir;

/// Start a mock server; every request body is forwarded to the
/// returned receiver for assertions.
pub fn spawn_mock_server() -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().expect("addr");
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || serve(listener, sender));
    (format!("http://{address}"), receiver)
}

/// Accept completions in order and record each request body.
fn serve(listener: TcpListener, requests: Sender<String>) {
    let calls = Arc::new(AtomicUsize::new(0));
    for stream in listener.incoming().flatten() {
        let request = read_request(stream.try_clone().expect("clone"));
        let call = calls.fetch_add(1, Ordering::SeqCst);
        if requests.send(request).is_err() {
            break;
        }
        respond(stream, call);
    }
}

/// Read one request; bodies here are small and arrive in one write.
fn read_request(mut stream: TcpStream) -> String {
    let mut buffer = [0u8; 16384];
    let read = stream.read(&mut buffer).unwrap_or_default();
    String::from_utf8_lossy(&buffer[..read]).to_string()
}

/// Reply with a tool call on the first request, then a final answer.
fn respond(mut stream: TcpStream, call: usize) {
    let body = match call {
        0 => {
            r#"{"choices":[{"message":{"role":"assistant","content":"reading","tool_calls":[{"id":"c1","type":"function","function":{"name":"read","arguments":"{\"path\":\"main.rs\"}"}}]},"finish_reason":"tool_calls"}]}"#
        }
        _ => {
            r#"{"choices":[{"message":{"role":"assistant","content":"done reading"},"finish_reason":"stop"}]}"#
        }
    };
    let _ = stream.write_all(
        format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .as_bytes(),
    );
    let _ = stream.flush();
}

/// Run the vox binary with piped stdin in a scratch project.
pub fn run_vox(project: &TempDir, base_url: &str, input: &str) -> (String, String) {
    run_vox_args(project, base_url, input, &[])
}

/// Run the vox binary with extra arguments.
pub fn run_vox_args(
    project: &TempDir,
    base_url: &str,
    input: &str,
    args: &[&str],
) -> (String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vox"))
        .current_dir(project.path())
        .env("OPENAI_API_KEY", "test-key")
        .env("VOX_DATA_DIR", project.path().join("data"))
        .env("VOX_CONFIG_DIR", project.path().join("config"))
        .env("VOX_LOG", "error")
        .args(["--base-url", base_url])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn vox");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait");
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// A scratch project with one readable file.
pub fn project() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    std::fs::write(dir.path().join("main.rs"), "alpha\nbeta\n").expect("seed");
    dir
}
