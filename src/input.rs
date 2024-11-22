// based on https://ratatui.rs/examples/apps/user_input/
// also TODO checkout tui-textarea


#[derive(Default, Debug)]
pub struct Input {
    pub content: String,
    pub char_idx: usize,
    // yank: String, // eventually? :)
}

impl Input {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            char_idx: 0,
        }
    }

    pub fn move_cursor(&mut self, move_right: bool) {
        let new_idx = if move_right {
            self.char_idx.saturating_add(1)
        } else {
            self.char_idx.saturating_sub(1)
        };

        self.char_idx = self.clamp_cursor(new_idx);
    }

    pub fn insert_char(&mut self, ch: char) {
        self.content.insert(self.byte_index(), ch);
        self.move_cursor(true);
    }

    pub fn delete_char(&mut self) {
        if self.char_idx != 0 {
            let before_idx = self.content.chars().take(self.char_idx - 1);
            let after_idx = self.content.chars().skip(self.char_idx);

            self.content = before_idx.chain(after_idx).collect();
            self.move_cursor(false);
        }
    }

    fn byte_index(&self) -> usize {
        self.content
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_idx)
            .unwrap_or(self.content.len())
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.content.chars().count())
    }
    
    // may be unnecessary if I always call Some(Input)
    pub fn reset(&mut self) {
        self.char_idx = 0;
        self.content = String::default();
    }
}
