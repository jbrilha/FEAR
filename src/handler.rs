use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct KeyHandler {
    mode: Mode,
    action: Action,
}

enum Action {
    None,
    Delete,
    Rename,
    Move,
}

enum Mode {
    Normal,
    Action(Action),
    // Input,
}

impl Default for KeyHandler {
    fn default() -> Self {
        Self {
            mode: Mode::Normal,
            action: Action::None,
        }
    }
}

impl KeyHandler {
    pub fn new() -> Self {
        KeyHandler::default()
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent, app: &mut App) -> AppResult<()> {
        let mode = std::mem::replace(&mut self.mode, Mode::Normal);

        self.mode = match mode {
            Mode::Action(action) => self.handle_action(app, action, key_event),
            Mode::Normal => self.handle_normal_mode(app, key_event),
            // Mode::Input => self.handle_input_mode(app, key_event),
        };

        Ok(())
    }

    fn handle_action(&mut self, app: &mut App, action: Action, key_event: KeyEvent) -> Mode {
        let mode = match action {
            Action::Delete => match key_event.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    app.delete_selection_or_cursor();
                    app.clear_msg();
                    Mode::Normal
                }
                _ => Mode::Normal,
            },
            Action::Rename => {
                self.handle_input(app, key_event)
                // Mode::Action(Action::Rename)
            }
            _ => Mode::Normal,
        };

        if matches!(mode, Mode::Normal) {
            app.clear_msg();
        }

        mode
    }

    fn handle_normal_mode(&mut self, app: &mut App, key_event: KeyEvent) -> Mode {
        let mut mode = Mode::Normal;

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
                mode = Mode::Action(Action::Delete);
                app.show_deletion_msg();
            }
            KeyCode::Char('r') => {
                mode = Mode::Action(Action::Rename);
                app.show_rename_msg();
            }
            _ => {}
        }

        mode
    }

    fn handle_input(&mut self, app: &mut App, key_event: KeyEvent) -> Mode {
        let mut mode = Mode::Action(Action::Rename);
        match key_event.code {
            KeyCode::Esc => mode = Mode::Normal,
            KeyCode::Char(c) => app.insert_char(c),
            KeyCode::Right => app.move_into(),
            KeyCode::Left => app.move_back(),
            KeyCode::Backspace => app.delete_char(),
            KeyCode::Enter => {
                mode = Mode::Normal;
                app.terminate_input();
            }
            _ => {}
        }

        mode
    }
}
