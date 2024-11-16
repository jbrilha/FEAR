use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct KeyHandler {
    action: Option<Action>,
}

enum Action {
    Delete,
}

impl Default for KeyHandler {
    fn default() -> Self {
        Self { action: None }
    }
}

impl KeyHandler {
    pub fn new() -> Self {
        KeyHandler::default()
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent, app: &mut App) -> AppResult<()> {
        match &self.action {
            Some(_) => self.handle_confirmation(app, key_event.code),
            None => {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.quit();
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        if key_event.modifiers == KeyModifiers::CONTROL {
                            app.quit();
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        app.move_into();
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        app.move_back();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.move_up();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.move_down();
                    }
                    KeyCode::Char(' ') => {
                        app.toggle_selection_on_cursor();
                        app.move_down();
                    }
                    KeyCode::Char('d') => {
                        self.action = Some(Action::Delete);
                        app.show_deletion_msg();
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_confirmation(&mut self, app: &mut App, key_code: KeyCode) {
        match self.action {
            Some(Action::Delete) => {
                match key_code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        app.delete_selection_or_cursor();
                    }
                    _ => {}
                }
            }
            None => ()
        }

        self.action = None;
        app.clear_msg();
    }
}
