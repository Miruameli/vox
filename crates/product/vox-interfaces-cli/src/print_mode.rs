//! Print mode: stream a single completion to standard output.

use std::io::Write;

use vox_config::{discover_project_config, load, LoadInputs};
use vox_domain::{CompletionRequest, Message, ModelProvider, StreamEvent};
use vox_provider_openai::{OpenAiClient, DEFAULT_BASE_URL};
use vox_shared::paths::config_dir;
use vox_shared::{VoxError, VoxResult};

use crate::cli::Cli;

/// Run one prompt through the configured provider and print the response.
///
/// Text deltas are written incrementally so output starts before the model
/// finishes. Secrets are never printed; only model text reaches stdout.
///
/// Print mode keeps no session - one prompt, streamed answer, exit - so
/// the session flags are rejected rather than silently ignored: a flag
/// that does nothing looks exactly like a session that was lost.
pub async fn run(cli: &Cli, prompt: &str) -> VoxResult<()> {
    reject_session_flags(cli)?;
    let config = resolve_config(cli)?;
    let api_key = read_api_key(&config)?;
    let base_url = config
        .general
        .base_url
        .clone()
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
    let client = OpenAiClient::new(base_url, api_key);
    let request =
        CompletionRequest::streaming(config.general.model.clone(), vec![Message::user(prompt)]);
    client
        .stream_completion(&request, &mut |event| match event {
            StreamEvent::Started { .. } => {}
            StreamEvent::Delta { text } => print!("{text}"),
            StreamEvent::Completed { .. } => {
                println!();
                let _ = std::io::stdout().flush();
            }
        })
        .await
}

/// Refuse the session flags: they belong to the interactive surface.
///
/// `--session` names a file to write and `--resume` continues a stored
/// conversation; neither means anything for one prompt that exits, so
/// both fail with the surface that does keep sessions.
fn reject_session_flags(cli: &Cli) -> VoxResult<()> {
    if cli.session.is_some() || cli.resume.is_some() {
        return Err(VoxError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "print mode keeps no session; run without -p to use --session/--resume",
        )));
    }
    Ok(())
}

/// Resolve configuration across all layers.
pub fn resolve_config(cli: &Cli) -> VoxResult<vox_config::Config> {
    let user_config = config_dir().join("config.toml");
    let user_config = user_config.is_file().then_some(user_config);
    let project_config = discover_project_config(&std::env::current_dir()?);
    let inputs = LoadInputs {
        user_config,
        project_config,
        env: std::env::vars().collect(),
        flags: cli.flag_overrides(),
    };
    load(&inputs)
}

/// Read the API key from the configured environment variable.
pub(crate) fn read_api_key(config: &vox_config::Config) -> VoxResult<String> {
    let name = &config.general.api_key_env;
    std::env::var(name).map_err(|_| VoxError::Credential {
        detail: format!("environment variable {name} is not set"),
    })
}
