//! Interactive-mode services: stdin lines, the Ctrl-C watcher, the
//! session store, and agent bootstrap.
//!
//! The REPL owns only parsing and rendering; every blocking or
//! environment-touching concern lives here so the loop stays a
//! plain async function that is easy to test.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc;
use vox_agent::{Agent, Limits};
use vox_config::Config;
use vox_engine_tools::ToolHarness;
use vox_provider_openai::{OpenAiClient, DEFAULT_BASE_URL};
use vox_shared::paths::data_dir;
use vox_shared::VoxResult;

use crate::print_mode::read_api_key;

/// Stream stdin lines; the channel closes at end of input.
pub fn spawn_line_reader() -> mpsc::UnboundedReceiver<String> {
    let (lines, lines_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Ok(result) = tokio::task::spawn_blocking(read_stdin_line).await {
            match result {
                Some(text) => {
                    if lines.send(text).is_err() {
                        break;
                    }
                }
                None => break,
            }
        }
    });
    lines_rx
}

/// Watch for Ctrl-C; each interrupt sets `cancelled` and sends a
/// notification so the idle loop can acknowledge it.
pub fn spawn_interrupt_watch(cancelled: &Arc<AtomicBool>) -> mpsc::UnboundedReceiver<()> {
    let (interrupts, interrupts_rx) = mpsc::unbounded_channel();
    let cancelled = Arc::clone(cancelled);
    tokio::spawn(async move {
        while tokio::signal::ctrl_c().await.is_ok() {
            cancelled.store(true, Ordering::SeqCst);
            let _ = interrupts.send(());
        }
    });
    interrupts_rx
}

/// One blocking stdin read; None at end of input.
fn read_stdin_line() -> Option<String> {
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => Some(line),
        Err(_) => None,
    }
}

/// Write one line to stdout and flush it.
pub fn emit(line: &str) {
    println!("{line}");
    let _ = std::io::stdout().flush();
}

/// Unix seconds, for default session ids.
pub fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

/// Session ids on disk, newest first.
pub fn sessions() -> Vec<String> {
    let mut found: Vec<(String, SystemTime)> = std::fs::read_dir(data_dir().join("sessions"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let id = path.file_stem()?.to_str()?.to_string();
            let written = entry.metadata().ok()?.modified().ok()?;
            (path.extension()?.to_str()? == "jsonl").then_some((id, written))
        })
        .collect();
    found.sort_by_key(|(_, written)| std::cmp::Reverse(*written));
    found.into_iter().map(|(id, _)| id).collect()
}

/// Build an agent for a session, seeding a resumed conversation.
pub fn build(config: &Config, session_id: &str, resumed: bool) -> VoxResult<Agent<OpenAiClient>> {
    let api_key = read_api_key(config)?;
    let path = vox_session::session_path(&data_dir(), session_id)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let client = OpenAiClient::new(
        config
            .general
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
        api_key,
    );
    let harness = ToolHarness::new(std::env::current_dir()?, &data_dir().join("undo.jsonl"))?;
    let agent = Agent::new(
        client,
        harness,
        &path,
        config.general.model.clone(),
        Limits::default(),
    )?;
    if resumed {
        let prior = agent.resume_messages()?;
        return Ok(agent.with_prior(prior));
    }
    Ok(agent)
}
