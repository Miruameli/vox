//! Execution bounds that keep the tool-use loop finite.

/// Bounds for one agent run.
///
/// The mandate forbids runaway agents: every loop has a hard turn cap
/// and a cap on consecutive tool failures, and both are part of the
/// public contract so callers can tune or audit them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Maximum provider round trips per run.
    pub max_turns: usize,
    /// Maximum consecutive tool failures before the run stops.
    pub max_consecutive_tool_errors: usize,
}

impl Limits {
    /// Build limits; both values must be at least one.
    pub fn new(max_turns: usize, max_consecutive_tool_errors: usize) -> Self {
        Self {
            max_turns: max_turns.max(1),
            max_consecutive_tool_errors: max_consecutive_tool_errors.max(1),
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::new(25, 3)
    }
}

/// Why a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinishReason {
    /// The model stopped without requesting more tools.
    ModelStopped,
    /// The turn cap was reached.
    TurnLimit,
    /// Consecutive tool failures reached the cap.
    ToolErrorLimit,
    /// The run was cancelled between actions.
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane_and_floored() {
        let limits = Limits::default();
        assert_eq!(limits.max_turns, 25);
        assert_eq!(limits.max_consecutive_tool_errors, 3);
        assert_eq!(Limits::new(0, 0), Limits::new(1, 1));
    }
}
