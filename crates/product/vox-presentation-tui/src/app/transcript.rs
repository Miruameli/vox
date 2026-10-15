//! Transcript entries: who spoke, what they said, and the detail a
//! tool entry can expand to.
//!
//! The transcript holds text, never rendered lines: the visible width
//! is only known at draw time, and the same entry renders differently
//! at 40 and at 200 columns. [`crate::render::wrap`] owns the wrapping
//! and turns entries into display lines.
//!
//! Entries are capped at [`TRANSCRIPT_LIMIT`]. The bound keeps memory
//! flat over a long session; the session file the agent writes is the
//! complete record, not this window.

/// Entries kept in view; the oldest are dropped past this bound.
pub const TRANSCRIPT_LIMIT: usize = 2000;

/// Who an entry came from, which decides how it is labelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// Text the user typed.
    User,
    /// Text the model produced.
    Assistant,
    /// A tool call that finished, successfully or not.
    Tool,
    /// A notice from the product: help, errors, commands.
    System,
}

impl EntryKind {
    /// Left-hand label rendered before the entry text.
    pub fn label(self) -> &'static str {
        match self {
            Self::User => "you",
            Self::Assistant => "vox",
            Self::Tool => "tool",
            Self::System => "vox",
        }
    }
}

/// One shown entry: who spoke, what they said, and optional detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Author of the entry.
    pub kind: EntryKind,
    /// Entry text, unwrapped; newlines are kept as written.
    pub text: String,
    /// Longer detail hidden until tool entries are expanded.
    pub detail: Option<String>,
}

impl Entry {
    /// Build an entry of `kind` carrying `text`.
    pub fn new(kind: EntryKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
            detail: None,
        }
    }

    /// Build an entry that carries hidden detail.
    pub fn with_detail(
        kind: EntryKind,
        text: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            text: text.into(),
            detail: Some(detail.into()),
        }
    }
}

/// The conversation shown by the view, oldest entry first.
#[derive(Debug, Default)]
pub struct Transcript {
    entries: Vec<Entry>,
}

impl Transcript {
    /// An empty transcript.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one entry, dropping the oldest past [`TRANSCRIPT_LIMIT`].
    pub fn push(&mut self, entry: Entry) {
        if self.entries.len() == TRANSCRIPT_LIMIT {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    /// Entries shown, oldest first.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Remove every entry.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
