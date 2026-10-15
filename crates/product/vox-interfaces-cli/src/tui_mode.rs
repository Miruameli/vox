//! Interactive mode: assemble the agent and hand it to the terminal UI.
//!
//! This module is the composition root. It resolves configuration
//! across all layers, reads the credential from the environment, and
//! builds the provider, the tool harness, the undo journal, and the
//! durable session. Everything after that belongs to
//! `vox-presentation-tui`, which never reads the environment itself -
//! so another host (an editor protocol surface, a test) can build the
//! same agent and show it differently.

use vox_agent::{Agent, Limits};
use vox_engine_tools::ToolHarness;
use vox_provider_openai::{OpenAiClient, DEFAULT_BASE_URL};
use vox_session::session_path;
use vox_shared::paths::data_dir;
use vox_shared::{VoxError, VoxResult};

use crate::cli::Cli;
use crate::print_mode;

/// Run the interactive surface until the user leaves.
pub async fn run(cli: &Cli) -> VoxResult<()> {
    let config = print_mode::resolve_config(cli)?;
    let api_key = print_mode::read_api_key(&config)?;
    let base_url = config
        .general
        .base_url
        .clone()
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
    let client = OpenAiClient::new(base_url, api_key);
    let root = std::env::current_dir()?;
    let journal = data_dir().join("undo.jsonl");
    let harness = ToolHarness::new(&root, &journal)?;
    let session = match &cli.resume {
        Some(id) => {
            // A resume names a session that must already exist: the
            // surface continues it instead of silently starting a
            // fresh one the user cannot find later.
            let file = session_path(&data_dir(), id)?;
            if !file.exists() {
                return Err(VoxError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("no session {id}"),
                )));
            }
            id.clone()
        }
        None => cli.session.clone().unwrap_or_else(session_id),
    };
    let session_file = session_path(&data_dir(), &session)?;
    let agent = Agent::new(
        client,
        harness,
        &session_file,
        config.general.model.clone(),
        Limits::default(),
    )?;
    // Seeds are a resume's job: the TUI continues a stored
    // conversation only for --resume. --session merely names the
    // file to write, so pairing it with an existing id starts
    // fresh there instead of silently continuing stale turns.
    let prior = if cli.resume.is_some() {
        agent.resume_messages()?
    } else {
        Vec::new()
    };
    let header = vox_presentation_tui::Header {
        model: config.general.model,
        session_id: session,
        project_root: root,
    };
    vox_presentation_tui::run(agent, header, prior).await
}

/// A session id that is safe as a file name and unique per launch.
///
/// The id is written to the data directory as `<id>.jsonl`, so it is
/// built from the launch time and the process id only: no path
/// separators, no spaces, nothing a shell would have to quote.
fn session_id() -> String {
    let launched = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    format!("tui-{launched}-{}", std::process::id())
}
