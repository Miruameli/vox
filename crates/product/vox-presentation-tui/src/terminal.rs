//! The terminal: raw mode, mouse capture, the alternate screen, and
//! restoring all three on every exit path.
//!
//! A TUI that leaves the terminal in raw mode takes the user's shell
//! with it, so the guard restores what it changed on drop - including
//! on a panic - and a partial setup undoes the steps that already
//! succeeded instead of leaving half a surface behind.

use std::io::{stdout, Stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};
use vox_shared::VoxResult;

/// The terminal surface: raw mode, the alternate screen, mouse capture.
pub struct Surface {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Surface {
    /// Enter raw mode, the alternate screen, and mouse capture.
    ///
    /// Each step is undone when a later one fails, so a failure never
    /// leaves the caller's terminal in a half-configured state.
    pub fn enter() -> VoxResult<Self> {
        enable_raw_mode()?;
        let mut screen_entered = false;
        let opened = (|| {
            execute!(stdout(), EnterAlternateScreen)?;
            screen_entered = true;
            execute!(stdout(), EnableMouseCapture)?;
            Terminal::new(CrosstermBackend::new(stdout()))
        })();
        match opened {
            Ok(terminal) => Ok(Self { terminal }),
            Err(error) => {
                let _ = execute!(stdout(), DisableMouseCapture);
                if screen_entered {
                    let _ = execute!(stdout(), LeaveAlternateScreen);
                }
                let _ = disable_raw_mode();
                Err(error.into())
            }
        }
    }

    /// Draw one frame into the surface.
    pub fn draw(&mut self, frame: impl FnOnce(&mut Frame<'_>)) -> VoxResult<()> {
        self.terminal.draw(frame)?;
        Ok(())
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        let _ = execute!(stdout(), DisableMouseCapture);
        let _ = execute!(stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}
