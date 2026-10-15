//! Recall: the submitted prompts the Up and Down keys step through.
//!
//! Stepping back stashes the line being typed as a draft, and stepping
//! forward past the newest prompt restores it, so recalling history
//! never destroys an unsent line.

/// Prompts kept for recall; the oldest is dropped past this bound.
const RECALL_LIMIT: usize = 200;

/// Prompts recalled with the Up and Down keys.
#[derive(Debug, Default)]
pub struct Recall {
    prompts: Vec<String>,
    /// Stepping position; `None` while editing a fresh line.
    index: Option<usize>,
    /// Text typed before stepping into history, restored on the way out.
    draft: String,
}

impl Recall {
    /// Empty recall.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a submitted prompt and return to editing a fresh line.
    pub fn push(&mut self, prompt: &str) {
        self.prompts.push(prompt.to_string());
        if self.prompts.len() > RECALL_LIMIT {
            self.prompts.remove(0);
        }
        self.index = None;
        self.draft.clear();
    }

    /// Step one prompt back, stashing the fresh line as the draft.
    pub fn previous(&mut self, draft: &str) -> Option<String> {
        let index = match self.index {
            None => {
                self.draft = draft.to_string();
                self.prompts.len().checked_sub(1)?
            }
            Some(index) => index.saturating_sub(1),
        };
        self.index = Some(index);
        self.prompts.get(index).cloned()
    }

    /// Step one prompt forward; past the newest it restores the draft.
    pub fn forward(&mut self) -> Option<String> {
        let index = self.index?;
        match self.prompts.get(index + 1) {
            Some(prompt) => {
                self.index = Some(index + 1);
                Some(prompt.clone())
            }
            None => {
                self.index = None;
                Some(std::mem::take(&mut self.draft))
            }
        }
    }

    /// Forget every recalled prompt and the stored draft.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
