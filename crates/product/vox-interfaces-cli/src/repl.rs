//! Interactive mode: the default entry point, a prompt loop over the
//! durable agent runtime.
//!
//! Each prompt runs one bounded agent turn-cycle; text, tool calls,
//! and tool results stream to the terminal as they happen, and every
//! event is already durable in the session file. Ctrl-C cancels the
//! run at the next safe point without ending the session.

use std::sync::atomic::Ordering;

use vox_agent::{FinishReason, LoopEvent};
use vox_shared::paths::data_dir;
use vox_shared::VoxResult;

use crate::cli::Cli;
use crate::print_mode::resolve_config;
use crate::runtime::{
    build, emit, prompt, sessions, spawn_interrupt_watch, spawn_line_reader, unix_secs,
};

/// One parsed line of REPL input.
enum Command {
    Help,
    Quit,
    Sessions,
    Session,
    Resume(String),
    Unknown(String),
    Prompt(String),
}

/// Run the interactive prompt loop until `/quit` or end of input.
pub async fn run(cli: &Cli) -> VoxResult<()> {
    let config = resolve_config(cli)?;
    std::fs::create_dir_all(data_dir())?;
    let mut session_id = cli
        .resume
        .clone()
        .or_else(|| cli.session.clone())
        .unwrap_or_else(|| format!("vox-{}", unix_secs()));
    let mut agent = build(&config, &session_id, cli.resume.is_some())?;
    let cancelled = agent.cancel_token();
    let mut interrupts = spawn_interrupt_watch(&cancelled);
    let mut lines = spawn_line_reader();
    emit(&format!(
        "vox {session_id} | model {} | /help lists commands",
        config.general.model
    ));
    loop {
        prompt();
        let line = tokio::select! {
            text = lines.recv() => match text {
                Some(text) => text,
                None => break,
            },
            _ = interrupts.recv() => {
                emit("(interrupting; the run stops at the next safe point)");
                continue;
            }
        };
        match parse_command(&line) {
            Command::Quit => break,
            Command::Help => emit("commands: /help  /session  /sessions  /resume <id>  /quit"),
            Command::Sessions => {
                let ids = sessions();
                emit(&if ids.is_empty() {
                    "(no sessions yet)".to_string()
                } else {
                    ids.join("\n")
                });
            }
            Command::Session => emit(&session_id),
            Command::Resume(id) if id.is_empty() => eprintln!("usage: /resume <session-id>"),
            Command::Resume(id) => match build(&config, &id, true) {
                Ok(next) => {
                    agent = next;
                    session_id = id;
                    emit(&format!("resumed session {session_id}"));
                }
                Err(error) => eprintln!("resume failed: {error}"),
            },
            Command::Unknown(line) => {
                eprintln!("unknown command: {line}; /help lists commands")
            }
            Command::Prompt(text) if text.is_empty() => {}
            Command::Prompt(text) => {
                let reason = agent.run(&text, &mut render_line).await;
                cancelled.store(false, Ordering::SeqCst);
                while interrupts.try_recv().is_ok() {}
                if let Err(error) = reason {
                    eprintln!("run failed: {error}");
                }
            }
        }
    }
    Ok(())
}

/// Print one loop event to the terminal.
fn render_line(event: LoopEvent) {
    match &event {
        LoopEvent::Text { text } => emit(text),
        LoopEvent::TurnStarted { turn } => emit(&format!("-- turn {turn} --")),
        LoopEvent::ToolStarted { name } => emit(&format!("[tool] {name}")),
        LoopEvent::ToolFinished { name, is_error } => emit(&format!(
            "[tool] {name} {}",
            if *is_error { "failed" } else { "ok" }
        )),
        LoopEvent::Finished { reason } => emit(&format!("-- {} --", finish_label(*reason))),
    }
}

/// One-line label for why a run ended.
fn finish_label(reason: FinishReason) -> &'static str {
    match reason {
        FinishReason::ModelStopped => "model stopped",
        FinishReason::TurnLimit => "turn limit reached",
        FinishReason::ToolErrorLimit => "tool error limit reached",
        FinishReason::Cancelled => "cancelled",
    }
}

/// Parse one input line; blank lines become empty prompts.
fn parse_command(line: &str) -> Command {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix('/') {
        let (head, arg) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
        return match head {
            "help" | "?" => Command::Help,
            "quit" | "exit" | "q" => Command::Quit,
            "sessions" => Command::Sessions,
            "session" => Command::Session,
            "resume" => Command::Resume(arg.trim().to_string()),
            _ => Command::Unknown(line.to_string()),
        };
    }
    Command::Prompt(line.to_string())
}
