//! End-to-end REPL checks against a mock OpenAI-compatible server:
//! one full tool cycle through the binary, and slash commands.

mod support;

use std::io::Write;

use support::{project, run_vox, run_vox_args, spawn_mock_server};

#[test]
fn a_prompt_runs_the_full_tool_cycle() {
    let (base_url, requests) = spawn_mock_server();
    let project = project();

    let (stdout, stderr) = run_vox(&project, &base_url, "read main.rs\n/session\n/quit\n");

    // The cycle reached the terminal: turn 1, the tool call, its
    // result, the second turn, and the run's end.
    assert!(stdout.contains("-- turn 1 --"), "{stdout}");
    assert!(stdout.contains("[tool] read"), "{stdout}");
    assert!(stdout.contains("[tool] read ok"), "{stdout}");
    assert!(stdout.contains("-- turn 2 --"), "{stdout}");
    assert!(stdout.contains("done reading"), "{stdout}");
    assert!(stdout.contains("-- model stopped --"), "{stdout}");
    assert!(!stdout.contains("[tool] read failed"), "{stdout}");
    assert!(!stderr.contains("run failed"), "{stderr}");

    // The banner's session id names a durable file holding the whole
    // cycle in order.
    let session = stdout
        .lines()
        .find_map(|line| line.strip_prefix("vox "))
        .and_then(|line| line.split(" | ").next())
        .expect("banner session id")
        .to_string();
    assert!(session.starts_with("vox-"), "{session}");
    let stored = std::fs::read_to_string(
        project
            .path()
            .join("data")
            .join("sessions")
            .join(format!("{session}.jsonl")),
    )
    .expect("session file");
    let kinds: Vec<String> = stored
        .lines()
        .filter_map(|line| {
            let value: serde_json::Value = serde_json::from_str(line).ok()?;
            value.get("type")?.as_str().map(str::to_string)
        })
        .collect();
    assert_eq!(
        kinds,
        vec!["message", "message", "tool_call", "tool_result", "message"]
    );

    // The read result carries hashlined content, and the request was
    // authenticated with the configured credential.
    assert!(stored.contains("644a9bc57c6063e2"), "{stored}");
    let first = requests
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("first request");
    assert!(first
        .to_lowercase()
        .contains("authorization: bearer test-key"));
}

#[test]
fn a_missing_key_fails_loudly_before_the_loop() {
    let (base_url, _requests) = spawn_mock_server();
    let project = project();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_vox"))
        .current_dir(project.path())
        .env_remove("OPENAI_API_KEY")
        .env("VOX_DATA_DIR", project.path().join("data"))
        .env("VOX_LOG", "error")
        .args(["--base-url", &base_url])
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(b"/quit\n")
        .expect("write");
    let output = child.wait_with_output().expect("wait");
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(stderr.contains("OPENAI_API_KEY"), "{stderr}");
}

#[test]
fn unknown_slash_commands_report_instead_of_prompting() {
    // A typo'd command must not spend a provider round trip.
    let (base_url, requests) = spawn_mock_server();
    let project = project();
    let (_, stderr) = run_vox(&project, &base_url, "/nope\n/help\n/quit\n");
    assert!(
        stderr.contains("unknown command: /nope; /help lists commands"),
        "{stderr}"
    );
    assert!(requests.try_recv().is_err());
}

#[test]
fn a_resumed_session_continues_the_conversation() {
    let (base_url, requests) = spawn_mock_server();
    let project = project();

    // First run: one read cycle, then exit.
    let (stdout, stderr) = run_vox_args(
        &project,
        &base_url,
        "read main.rs\n/quit\n",
        &["--session", "s1"],
    );
    assert!(stdout.contains("-- model stopped --"), "{stdout}{stderr}");

    // Second run on the same session id: the prior conversation is
    // in the provider request before the new prompt.
    let (stdout, stderr) = run_vox_args(
        &project,
        &base_url,
        "what did it say\n/quit\n",
        &["--resume", "s1"],
    );
    assert!(stdout.contains("vox s1 | "), "{stdout}{stderr}");
    assert!(stdout.contains("-- model stopped --"), "{stdout}{stderr}");

    // Skip the first run's two requests; inspect the resumed one.
    for _ in 0..2 {
        assert!(requests
            .recv_timeout(std::time::Duration::from_secs(5))
            .is_ok());
    }
    let resumed = requests
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("resumed request");
    assert!(resumed.contains("read main.rs"), "{resumed}");
    assert!(resumed.contains("alpha"), "{resumed}");
    assert!(resumed.contains("what did it say"), "{resumed}");
}
