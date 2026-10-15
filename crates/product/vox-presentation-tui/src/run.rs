//! The event loop: keys in, frames out, the agent on its own task.
//!
//! The loop blocks only on `crossterm` event polling with a timeout,
//! and drains the worker channel every round. A turn that takes
//! minutes therefore never freezes the terminal: the indicator keeps
//! animating, scrolling keeps working, and the interrupt key stays
//! live while the model is thinking.

use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind, MouseEventKind};
use tokio::sync::mpsc;
use vox_agent::Agent;
use vox_domain::message::Message;
use vox_domain::provider::ModelProvider;
use vox_shared::VoxResult;

use crate::app::{handle_key, App, AppMsg};
use crate::render::draw;
use crate::terminal::Surface;
use crate::worker::Worker;
use crate::Header;

/// How long one idle poll waits for input.
///
/// Short enough that the running indicator animates smoothly, long
/// enough that an idle loop does not spin the CPU.
const POLL: Duration = Duration::from_millis(40);

/// Wrap a failure of the blocking task as an ordinary error.
fn task_failed(error: tokio::task::JoinError) -> vox_shared::VoxError {
    vox_shared::VoxError::Io(std::io::Error::other(error.to_string()))
}

/// Run the interactive surface until the user leaves.
///
/// `agent` arrives assembled by the composition root; the surface never
/// reads the environment, the configuration, or the session file.
/// `prior` seeds the conversation with turns a resumed session already
/// holds, so the first prompt continues them. The render loop runs on
/// a blocking thread so a caller that runs this on a single-threaded
/// runtime still gets a responsive surface.
pub async fn run<P: ModelProvider + 'static>(
    agent: Agent<P>,
    header: Header,
    prior: Vec<Message>,
) -> VoxResult<()> {
    let cancel = agent.cancel_token();
    let (events, inbox) = mpsc::unbounded_channel();
    let worker = Worker::spawn(agent, events, cancel, prior);
    tokio::task::spawn_blocking(move || block_on_input(header, inbox, worker))
        .await
        .unwrap_or_else(|error| Err(task_failed(error)))
}

/// Drive frames and keys until the user asks to leave.
fn block_on_input(
    header: Header,
    mut inbox: mpsc::UnboundedReceiver<AppMsg>,
    worker: Worker,
) -> VoxResult<()> {
    let mut app = App::new(header);
    let mut surface = Surface::enter()?;
    while !app.quit {
        while let Ok(message) = inbox.try_recv() {
            app.apply(message);
        }
        surface.draw(|frame| draw(frame, &mut app))?;
        if !event::poll(POLL)? {
            app.apply(AppMsg::Tick);
            continue;
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match handle_key(&mut app, key) {
                crate::app::Action::Nothing | crate::app::Action::Redraw => {}
                crate::app::Action::Submit(prompt) => worker.submit(prompt)?,
                crate::app::Action::Cancel => worker.cancel(),
                crate::app::Action::Quit => app.quit = true,
            },
            Event::Mouse(scroll) => match scroll.kind {
                MouseEventKind::ScrollUp => crate::app::scroll_page(&mut app, true),
                MouseEventKind::ScrollDown => crate::app::scroll_page(&mut app, false),
                _ => {}
            },
            // A resize is picked up by the next draw, which sizes the
            // frame from the terminal itself.
            _ => {}
        }
    }
    Ok(())
}
