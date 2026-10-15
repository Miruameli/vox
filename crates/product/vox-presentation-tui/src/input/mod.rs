//! The prompt line: one editable line of text with a cursor.
//!
//! Editing is modelled on characters, not bytes, so multibyte input
//! deletes one visible character per backspace. [`InputBuffer::view`]
//! returns the horizontal window that keeps the cursor on screen; the
//! renderer draws exactly what it returns and positions the terminal
//! cursor inside it. [`Recall`] holds the prompts the Up and Down keys
//! step through.

pub mod recall;

pub use recall::Recall;

/// One editable line of text with a cursor.
#[derive(Debug, Default, Clone)]
pub struct InputBuffer {
    characters: Vec<char>,
    cursor: usize,
}

impl InputBuffer {
    /// An empty buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert one character at the cursor.
    pub fn insert(&mut self, character: char) {
        self.characters.insert(self.cursor, character);
        self.cursor += 1;
    }

    /// Replace the text and park the cursor after the end.
    pub fn set_text(&mut self, text: &str) {
        self.characters = text.chars().collect();
        self.cursor = self.characters.len();
    }

    /// Delete the character before the cursor.
    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        self.characters.remove(self.cursor);
    }

    /// Delete the character at the cursor.
    pub fn delete(&mut self) {
        if self.cursor >= self.characters.len() {
            return;
        }
        self.characters.remove(self.cursor);
    }

    /// Move the cursor one character left.
    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Move the cursor one character right.
    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.characters.len());
    }

    /// Move the cursor to the start of the line.
    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    /// Move the cursor to the end of the line.
    pub fn move_end(&mut self) {
        self.cursor = self.characters.len();
    }

    /// Remove all text and park the cursor at the start.
    pub fn clear(&mut self) {
        self.characters.clear();
        self.cursor = 0;
    }

    /// Whether the buffer holds no text.
    pub fn is_empty(&self) -> bool {
        self.characters.is_empty()
    }

    /// Buffered text.
    pub fn text(&self) -> String {
        self.characters.iter().collect()
    }

    /// Cursor position, counted in characters from the start.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Take the text out, leaving an empty buffer.
    pub fn take(&mut self) -> String {
        self.cursor = 0;
        self.characters.drain(..).collect()
    }

    /// The visible window at `width`, and the cursor inside it.
    ///
    /// The window scrolls only when the cursor would fall outside it,
    /// so short prompts render from the first character and long ones
    /// follow the cursor.
    pub fn view(&self, width: usize) -> (String, usize) {
        let width = width.max(1);
        if self.characters.len() <= width {
            let text: String = self.characters.iter().collect();
            return (text, self.cursor.min(width));
        }
        let start = if self.cursor + 1 > width {
            (self.cursor + 1 - width).min(self.characters.len() - width)
        } else {
            0
        };
        let end = (start + width).min(self.characters.len());
        let text: String = self.characters[start..end].iter().collect();
        let cursor = self.cursor - start;
        (text, cursor)
    }
}
