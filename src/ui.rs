use std::{
    any, fs, io, path::{Path, PathBuf}
};

use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};

use crate::{app::App, sorter::Sorter};

const SORTER: Sorter = Sorter::DirsFirst;
const MARK: &str = "  ";

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples

    frame.render_widget(
        title_bar(&app.focus.path),
        app.titlebar_layout[0],
    );

    frame.render_widget(
        info_bar(&app.focus.path),
        app.titlebar_layout[1],
    );

    frame.render_widget(
        parent_pane(app, app.parent_layout.width.into()),
        app.parent_layout,
    );

    frame.render_widget(
        focus_pane(app, app.focus_layout.width.into()),
        app.focus_layout,
    );

    frame.render_widget(
        preview_pane(app, app.preview_layout.width.into()),
        app.preview_layout,
    );
}

fn title_bar(path: &Path) -> Paragraph {
    Paragraph::new("")
        .block(
            Block::default()
                .title(format!("— {} ", path.to_string_lossy().into_owned()))
                .borders(Borders::TOP),
        )
        .style(Style::default().fg(Color::Cyan))
}

fn info_bar(_path: &Path) -> Paragraph {
    Paragraph::new("")
        .block(
            Block::default()
                .title(" placeholder | will be dir info? ")
                .title_alignment(Alignment::Right)
                .borders(Borders::TOP),
        )
        .style(Style::default().fg(Color::Cyan))
}

fn parent_pane(app: &App, width: usize) -> Paragraph {
    let paths: Vec<Line> = app.parent_dir.as_ref().map_or_else(
        || Vec::new(),
        |dir| {
            dir.contents
                .iter()
                .map(|path| format_line(app, path.to_path_buf(), width, PaneContext::Parent))
                .collect()
        },
    );

    Paragraph::new(Text::from(paths))
        .block(Block::default())
        .style(Style::default().fg(Color::Cyan))
}

fn focus_pane(app: &App, width: usize) -> Paragraph {
    let paths: Vec<Line> = app
        .focus
        .contents
        .iter()
        .map(|path| format_line(app, path.to_path_buf(), width, PaneContext::Focus))
        .collect();

    let display = if paths.is_empty() {
        Text::from(format!("{:<width$}", "Nothing to see here"))
            .style(Style::default().fg(Color::Red).bg(Color::Black))
    } else {
        Text::from(paths)
    };

    Paragraph::new(display)
        .block(Block::default().padding(Padding::symmetric(1, 0)))
        .style(Style::default().fg(Color::Cyan))
}

fn preview_pane(app: &App, width: usize) -> Paragraph {
    let preview = match &app.cursor {
        Some(cursor) => {
            let selected = Path::new(&cursor);
            if selected.is_dir() {
                let sub_paths: Vec<Line> = match fs::read_dir(selected) {
                    Ok(dir) => {
                        let mut entries: Vec<_> = dir.filter_map(|p| p.ok()).collect();

                        entries.sort_by(|a, b| SORTER.entries(a, b));

                        entries
                            .into_iter()
                            .map(|entry| {
                                format_line(app, entry.path(), width, PaneContext::Preview)
                            })
                            .collect()
                    }
                    Err(err) => match err.kind() {
                        io::ErrorKind::PermissionDenied => {
                            vec![Line::from("Permission denied!".to_string())]
                        }
                        _ => vec![Line::from("Unknown error".to_string())],
                    },
                };

                if sub_paths.is_empty() {
                    Text::from("Empty...")
                } else {
                    Text::from(sub_paths)
                    // sub_paths.sort();
                    // sub_paths.join("\n")
                }
            } else if selected.is_file() {
                Text::from(
                    fs::read_to_string(selected)
                        .unwrap_or_else(|_| "Failed to parse whatever that is".to_string()),
                )
            } else {
                Text::from("Neither a file nor a directory...")
            }
        }
        None => Text::from("nope"),
    };

    Paragraph::new(preview)
        .block(Block::default())
        .style(Style::default().fg(Color::Cyan))
}

enum PaneContext {
    Parent,
    Focus,
    Preview,
}

fn format_line(app: &App, path: PathBuf, width: usize, ctx: PaneContext) -> Line {
    let mut fg_color = Color::White;
    let mut bg_color = Color::Reset;
    let mut basename = path.file_name().unwrap().to_string_lossy().into_owned();
    if path.is_dir() {
        fg_color = Color::LightMagenta;
    } else if path.is_symlink() {
        fg_color = Color::Cyan;
    }

    if app.selections.values().any(|set| set.contains(&path)) {
        if matches!(ctx, PaneContext::Focus) {
            fg_color = Color::Yellow;
        }
        basename = MARK.to_owned() + &basename;
    }
    if Some(&path) == app.cursor.as_ref()
        || (matches!(ctx, PaneContext::Parent) && app.focus.path == path)
    {
        bg_color = fg_color;
        fg_color = Color::Black;
    }

    let padded_name = format!("{:<width$}", basename);
    let style = Style::default().fg(fg_color).bg(bg_color);
    Line::from(padded_name).style(style)
}
