//! The view state: a reducer over keys and agent events.
//!
//! Every transition is a plain function of the previous state plus one
//! key press ([`keys::handle_key`]) or one [`AppMsg`] from the agent
//! bridge ([`events::apply`]). No terminal, provider, or clock is
//! reachable here, so the state machine is testable without a tty: the
//! renderer reads the state, the event loop writes to it.

pub mod commands;
pub mod events;
pub mod keys;
pub mod transcript;

pub use commands::{Command, Parsed};
pub use events::{reason_text, AppMsg};
pub use keys::{handle_key, scroll_page, Action};
pub use transcript::{Entry, EntryKind, Transcript};

use crate::input::{InputBuffer, Recall};
use crate::Header;

/// What the run is doing right now.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RunState {
    /// No run is executing.
    #[default]
    Idle,
    /// A turn is executing; `tool` names the running tool, if any.
    Running {
        /// 1-based turn number.
        turn: usize,
        /// Tool currently executing, when one is.
        tool: Option<String>,
    },
}

/// The view state.
pub struct App {
    /// Host facts shown in the header.
    pub(crate) header: Header,
    /// Shown conversation.
    pub(crate) transcript: Transcript,
    /// Line being edited.
    pub(crate) input: InputBuffer,
    /// Prompts recalled by the Up and Down keys.
    pub(crate) recall: Recall,
    /// Whether a run is executing.
    pub(crate) state: RunState,
    /// Why the last run ended; shown until the next prompt.
    pub(crate) last_reason: Option<vox_agent::FinishReason>,
    /// Set while a cancel is requested but not yet observed.
    pub(crate) cancelling: bool,
    /// Lines the viewport is scrolled back from the tail; 0 follows it.
    pub(crate) scroll_from_tail: usize,
    /// Visible body height in lines, refreshed by the renderer.
    pub(crate) body_height: usize,
    /// Whether tool entries show their hidden detail.
    pub(crate) expand_tools: bool,
    /// Whether tool entries are shown at all.
    pub(crate) show_tools: bool,
    /// Frame counter that animates the running indicator.
    pub(crate) spinner: usize,
    /// Whether the user asked to leave; the event loop exits on it.
    pub(crate) quit: bool,
}

impl App {
    /// A state showing one greeting and nothing else.
    pub fn new(header: Header) -> Self {
        let mut app = Self {
            header,
            transcript: Transcript::new(),
            input: InputBuffer::new(),
            recall: Recall::new(),
            state: RunState::Idle,
            last_reason: None,
            cancelling: false,
            scroll_from_tail: 0,
            body_height: 0,
            expand_tools: false,
            show_tools: true,
            spinner: 0,
            quit: false,
        };
        app.notice("Vox ready. Type a prompt and press Enter. /help lists keys and commands.");
        app
    }

    /// Append one message from the agent bridge.
    pub fn apply(&mut self, message: AppMsg) {
        events::apply(self, message);
    }

    /// Append a system notice to the transcript.
    pub(crate) fn notice(&mut self, text: impl Into<String>) {
        self.transcript.push(Entry::new(EntryKind::System, text));
    }

    /// Entries shown in the transcript, oldest first.
    pub fn entries(&self) -> &[Entry] {
        self.transcript.entries()
    }

    /// The prompt line being edited.
    pub fn input(&self) -> &InputBuffer {
        &self.input
    }

    /// The prompt line being edited, mutably.
    pub fn input_mut(&mut self) -> &mut InputBuffer {
        &mut self.input
    }

    /// What the run is doing right now.
    pub fn run_state(&self) -> &RunState {
        &self.state
    }

    /// Whether a cancel was requested and not yet observed.
    pub fn is_cancelling(&self) -> bool {
        self.cancelling
    }

    /// Whether tool activity is shown.
    pub fn shows_tools(&self) -> bool {
        self.show_tools
    }

    /// Whether tool entries show their detail.
    pub fn expands_tools(&self) -> bool {
        self.expand_tools
    }

    /// Visible body height in lines.
    pub fn body_height(&self) -> usize {
        self.body_height
    }

    /// Record the visible body height in lines.
    ///
    /// The renderer calls this each frame. It is public so a test can
    /// set the geometry without a terminal.
    pub fn set_body_height(&mut self, height: usize) {
        self.body_height = height;
    }

    /// Lines the viewport is scrolled back from the tail.
    pub fn scroll_from_tail(&self) -> usize {
        self.scroll_from_tail
    }

    /// Frame counter that animates the running indicator.
    pub fn spinner(&self) -> usize {
        self.spinner
    }
}
