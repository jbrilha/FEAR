use std::{fs, io, path::Path};

use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, Widget},
    Frame,
};

use crate::app::App;

const TAB: &str = " * ";

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples

    frame.render_widget(
        title_bar(&app.current_dir.path),
        // .centered(),
        app.titlebar_layout[0],
    );

    frame.render_widget(
        parent_pane(app, app.explorer_layout[0].width.into()),
        app.explorer_layout[0],
    );

    frame.render_widget(
        main_pane(app, app.explorer_layout[1].width.into()),
        app.explorer_layout[1]
    );

    let selected = Path::new(&app.cursor);
    let preview = if selected.is_dir() {
        let mut sub_paths: Vec<String> = match fs::read_dir(selected) {
            Ok(dir) => dir
                .filter_map(|p| {
                    p.ok()
                        .and_then(|entry| entry.file_name().into_string().ok())
                })
                .collect(),
            Err(err) => match err.kind() {
                io::ErrorKind::PermissionDenied => vec!["Permission denied!".to_string()],
                _ => vec!["Unknown error".to_string()],
            },
        };

        if sub_paths.is_empty() {
            "Empty...".to_string()
        } else {
            sub_paths.sort();
            sub_paths.join("\n")
        }
    } else if selected.is_file() {
        fs::read_to_string(selected)
            .unwrap_or_else(|_| "Failed to parse whatever that is".to_string())
    } else {
        "Neither a file nor a directory...".to_string()
    };

    frame.render_widget(
        Paragraph::new(preview)
            .block(
                Block::bordered()
                    .title(" Preview ")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Cyan)),
        // .centered(),
        app.explorer_layout[2],
    );
}

fn title_bar(path: &Path) -> Paragraph {
    Paragraph::new(path.to_string_lossy().into_owned())
        .block(
            Block::bordered()
                .title(" Title ")
                .title_alignment(Alignment::Left)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan))
}

fn main_pane(app: &App, width: usize) -> Paragraph {
    let paths: Vec<Line> = app
        .current_dir
        .contents
        .iter()
        .map(|path| {
            let mut fg_color = Color::White;
            let mut bg_color = Color::Reset;
            let mut basename = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() {
                fg_color = Color::LightMagenta;
            } else if path.is_symlink() {
                fg_color = Color::Cyan;
            }

            if app.current_selections().contains(path) {
                fg_color = Color::Yellow;
                basename = TAB.to_owned() + &basename;
            }
            if *path == app.cursor {
                bg_color = fg_color;
                fg_color = Color::Black;
            }
            
            let padded_name = format!("{:<width$}", basename);
            let style = Style::default().fg(fg_color).bg(bg_color);
            Line::from(padded_name).style(style)
        })
        .collect();

    Paragraph::new(Text::from(paths))
        .block(
            Block::bordered()
                .title(" Main ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan))
}

fn parent_pane(app: &App, width: usize) -> Paragraph {
    let paths: Vec<Line> = app.parent_dir.as_ref().map_or_else(
        || Vec::new(),
        |dir| {
            dir.contents
                .iter()
                .map(|path| {
                    let mut fg_color = Color::White;
                    let mut bg_color = Color::Reset;
                    let basename = path.file_name().unwrap().to_string_lossy().into_owned();
                    if path.is_dir() {
                        fg_color = Color::LightMagenta;
                    } else if path.is_symlink() {
                        fg_color = Color::Cyan;
                    }

                    // if app.current_selections().contains(path) {
                    //     fg_color = Color::Yellow;
                    //     // bg_color = Color::Black;
                    //     basename = TAB.to_owned() + &basename;
                    // }
                    if *path == app.current_dir.path {
                        bg_color = fg_color;
                        fg_color = Color::Black;
                    }
                    let padded_name = format!("{:<width$}", basename);
                    let style = Style::default().fg(fg_color).bg(bg_color);
                    Line::from(padded_name).style(style)
                })
                .collect()
        },
    );

    Paragraph::new(Text::from(paths))
        .block(
            Block::bordered()
                .title(" Main ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan))
}
