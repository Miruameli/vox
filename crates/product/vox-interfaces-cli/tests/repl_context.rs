//! Continuity checks: the conversation survives across processes
//! (--resume) and across prompts inside one REPL session.

mod support;

use support::{project, run_vox, run_vox_args, spawn_mock_server};

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

#[test]
fn a_second_prompt_in_one_process_keeps_the_conversation() {
    // Interactive sessions call run() per line; the second prompt
    // must reach the provider with the first turn's context.
    let (base_url, requests) = spawn_mock_server();
    let project = project();
    let (stdout, stderr) = run_vox(&project, &base_url, "read main.rs\nand now?\n/quit\n");

    let turns = stdout.contains("-- turn 1 --") && stdout.contains("-- turn 2 --");
    assert!(turns && stdout.contains("done reading"), "{stdout}");
    assert!(!stderr.contains("run failed"), "{stderr}");

    // Three provider round trips: read cycle (two) plus the follow-up.
    let mut seen = Vec::new();
    while let Ok(request) = requests.recv_timeout(std::time::Duration::from_secs(5)) {
        seen.push(request);
    }
    assert_eq!(seen.len(), 3, "{} requests", seen.len());
    let follow_up = seen.last().expect("follow-up request");
    assert!(follow_up.contains("read main.rs"), "{follow_up}");
    assert!(follow_up.contains("alpha"), "{follow_up}");
    assert!(follow_up.contains("and now?"), "{follow_up}");
}
