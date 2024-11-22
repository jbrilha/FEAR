use std::io;

use handler::KeyHandler;
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::{
    app::{App, AppResult},
    event::{Event, EventHandler},
    tui::Tui,
};

pub mod app;
pub mod directory_entry;
pub mod event;
pub mod file_entry;
pub mod filesystem_entry;
pub mod handler;
pub mod input;
pub mod sorter;
pub mod tui;
pub mod ui;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let size = terminal.size();
    let events = EventHandler::new(250);
    let mut keys = KeyHandler::new();
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Create an application.
    let mut app = App::new(size.unwrap());

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.

        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => keys.handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(width, height) => {
                app.generate_layout(Rect::new(0, 0, width, height));
            }
        }
    }

    // Exit the user interface.
    tui.exit()?;
    eprintln!("{}", app.focus_dir.path.display());
    Ok(())
}
