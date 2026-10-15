//! End-to-end print-mode test against a local mock SSE server.
//!
//! No real network access occurs: the mock server listens on loopback and
//! serves a canned streaming chat completion.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;

/// Canned streaming body: two deltas, a usage chunk, then the sentinel.
const SSE_BODY: &str = concat!(
    "data: {\"model\":\"mock-model\",\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n",
    "data: {\"model\":\"mock-model\",\"choices\":[{\"delta\":{\"content\":\" world\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":3,\"completion_tokens\":2}}\n\n",
    "data: [DONE]\n\n",
);

/// Start the mock server and return its base URL and the request receiver.
fn spawn_mock_server() -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().expect("local addr");
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0u8; 4096];
            let read = stream.read(&mut buffer).unwrap_or(0);
            let request = String::from_utf8_lossy(&buffer[..read]).to_string();
            let _ = sender.send(request);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                SSE_BODY.len(),
                SSE_BODY
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });
    (format!("http://{address}/v1"), receiver)
}

/// Run the CLI binary with the given arguments and environment.
fn run_cli(base_url: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vox"))
        .args(["-p", "hi", "--base-url", base_url])
        .env("OPENAI_API_KEY", "test-key")
        .output()
        .expect("run cli")
}

#[test]
fn print_mode_streams_mock_completion_to_stdout() {
    let (base_url, request_rx) = spawn_mock_server();
    let output = run_cli(&base_url);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello world\n");

    let request = request_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("request");
    assert!(
        request.starts_with("POST /v1/chat/completions"),
        "request line: {request}"
    );
    assert!(request
        .to_lowercase()
        .contains("authorization: bearer test-key"));
    assert!(request.contains("\"stream\":true"));
}

#[test]
fn print_mode_fails_without_credential() {
    let (base_url, _request_rx) = spawn_mock_server();
    let output = Command::new(env!("CARGO_BIN_EXE_vox"))
        .args(["-p", "hi", "--base-url", &base_url])
        .env_remove("OPENAI_API_KEY")
        .output()
        .expect("run cli");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("credential"), "stderr: {stderr}");
    assert!(!stderr.contains("test-key"), "credential leaked: {stderr}");
}

#[test]
fn config_subcommand_prints_resolved_settings() {
    let output = Command::new(env!("CARGO_BIN_EXE_vox"))
        .args(["config", "--model", "mock-model"])
        .env_remove("VOX_MODEL")
        .output()
        .expect("run cli");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("model      = mock-model"),
        "stdout: {stdout}"
    );
}
