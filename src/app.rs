use std::error;

use ratatui::layout::{Constraint, Direction, Layout, Rect, Size};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub counter: u8,
    pub inner_layout: Vec<Rect>,
    pub outer_layout: Vec<Rect>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            inner_layout: vec![Rect::default()],
            outer_layout: vec![Rect::default()],
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(size: Size) -> Self {
        let mut app = Self::default();
        app.generate_layout(Rect::new(0, 0, size.width, size.height));
        app
    }

    pub fn generate_layout(&mut self, size: Rect) {
            // panic!("{}", size.area());
        self.outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(15),
                    Constraint::Percentage(35),
                    Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(size)
            .to_vec();

        // self.inner_layout = Layout::default()
        //     .direction(Direction::Horizontal)
        //     .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
        //     .split(self.outer_layout[1])
        //     .to_vec();
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }
}
