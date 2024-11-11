use std::{fs, io, path::Path};

use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Paragraph},
    Frame,
};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    frame.render_widget(
        Paragraph::new(format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Counter: {}",
            app.counter
        ))
        .block(
            Block::bordered()
                .title("Template")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .centered(),
        app.outer_layout[0],
    );

    let mut paths: Vec<String> = fs::read_dir("./")
        .unwrap()
        .filter_map(|p| {
            p.ok()
                .and_then(|entry| entry.file_name().into_string().ok())
        })
        .collect();

    paths.sort();
    let path_list_str = paths.join("\n");

    frame.render_widget(
        Paragraph::new(path_list_str)
            .block(
                Block::bordered()
                    .title("Main")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .centered(),
        app.outer_layout[1],
    );

    let selected = Path::new(paths.get(0).unwrap());
    let preview = if selected.is_dir() {
        let mut sub_paths: Vec<String> = fs::read_dir(selected)
            .unwrap()
            .filter_map(|p| {
                p.ok()
                    .and_then(|entry| entry.file_name().into_string().ok())
            })
            .collect();

        if sub_paths.is_empty() {
            "Empty...".to_string()
        } else {
            sub_paths.sort();
            sub_paths.join("\n")
        }
    } else if selected.is_file() {
        fs::read_to_string(selected).expect("Should have been able to read :(")
    } else {
        todo!("Neither a file nor a directory...");
    };

    frame.render_widget(
        Paragraph::new(preview)
            .block(
                Block::bordered()
                    .title("Preview")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .centered(),
        app.outer_layout[2],
    );
}
