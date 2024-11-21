use std::{
    fs, io,
    path::{Path, PathBuf},
};

use lopdf::Document;
use ratatui::{
    layout::{Alignment, Constraint, Margin, Position},
    style::{Color, Style},
    symbols::scrollbar::{self, Set},
    text::{Line, Text},
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
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
    render_title_bar(frame, app);

    render_info_bar(frame, app);

    match &app.parent_dir {
        Some(_) => render_parent_pane(frame, app),
        None => {}
    }

    render_focus_pane(frame, app);

    render_preview_pane(frame, app);

    if let Some(m) = &app.message {
        match &app.input {
            Some(input) => {
                frame.render_widget(
                    Paragraph::new(Text::from(m.to_string() + &input.content))
                        .block(Block::default().borders(Borders::TOP))
                        .style(Style::default().fg(Color::Cyan)),
                    app.message_layout,
                );
                frame.set_cursor_position(Position::new(
                    app.message_layout.x + (m.chars().count() + input.char_idx) as u16,
                    app.message_layout.y + 1,
                ));
            }
            None => {
                frame.render_widget(
                    Paragraph::new(Text::from(m.to_string()))
                        .block(Block::default().borders(Borders::TOP))
                        .style(Style::default().fg(Color::Cyan)),
                    app.message_layout,
                );
            }
        }
    }

    // if let Some(cursor) = &app.app_cursor {
    //     frame.render_widget(
    //         Paragraph::new(Text::from(
    //             cursor.idx.to_string()
    //                 + &app
    //                     .path_stack
    //                     .iter()
    //                     .map(|pb| format!("{} : {}", pb.to_string_lossy(), 0))
    //                     .collect::<Vec<String>>()
    //                     .join(" | ")
    //                 + "\n"
    //                 + &app
    //                     .forward_stack
    //                     .iter()
    //                     .map(|pb| format!("{} : {}", pb.to_string_lossy(), 0))
    //                     .collect::<Vec<String>>()
    //                     .join(" | "),
    //         ))
    //         .block(Block::default().borders(Borders::TOP))
    //         .style(Style::default().fg(Color::Cyan)),
    //         app.message_layout,
    //     );
    // }

    // frame.render_widget(
    //     Paragraph::new(Text::from(format!("{} @ ", app.app_cursor.expect("fds").idx.to_string()) +
    // &app.path_stack
    //     .iter()
    //     .map(|(pb, idx)| format!("{} : {}", pb.to_string_lossy(), idx))
    //     .collect::<Vec<String>>().join(" | ")
    //     ))
    //     .block(Block::default().borders(Borders::TOP))
    //     .style(Style::default().fg(Color::Cyan)),
    //     app.message_layout,
    // );
}

fn render_title_bar(frame: &mut Frame, app: &App) {
    let path = &app.focus_dir.path;

    let p = Paragraph::new("")
        .block(
            Block::default()
                .title(format!("— {} ", path.to_string_lossy().into_owned()))
                .borders(Borders::TOP),
        )
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(p, app.titlebar_layout[0]);
}

fn render_info_bar(frame: &mut Frame, app: &App) {
    let scroll_pos = match &app.app_cursor {
        Some(c) => c.idx,
        None => 0,
    };
    let p = Paragraph::new("")
        .block(
            Block::default()
                .title(format!("{} | {}", scroll_pos, app.focus_layout.height))
                .title_alignment(Alignment::Right)
                .borders(Borders::TOP),
        )
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(p, app.titlebar_layout[1]);
}

fn render_parent_pane(frame: &mut Frame, app: &App) {
    let paths: Vec<Line> = app.parent_dir.as_ref().map_or_else(
        || Vec::new(),
        |dir| {
            dir.contents
                .iter()
                .map(|path| {
                    format_line(
                        app,
                        path.to_path_buf(),
                        app.parent_layout.width.into(),
                        PaneContext::Parent,
                    )
                })
                .collect()
        },
    );

    let p = Paragraph::new(Text::from(paths))
        .block(Block::default())
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(p, app.parent_layout)
}

fn render_focus_pane(frame: &mut Frame, app: &App) {
    let width = app.focus_layout.width.into();
    let paths: Vec<Line> = app
        .focus_dir
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

    let scroll_pos = match &app.app_cursor {
        Some(c) => {
            let idx = c.idx;
            let height = app.focus_layout.height as usize;
            let lines = display.lines.len();

            if lines <= height {
                0
            } else {
                let middle = height / 2;

                if idx >= lines.saturating_sub(middle) {
                    lines.saturating_sub(height)
                } else {
                    idx.saturating_sub(middle)
                }
            }
        }
        None => 0,
    };

    let p = Paragraph::new(display)
        .block(Block::default().padding(Padding::symmetric(1, 0)))
        .scroll((scroll_pos as u16, 0))
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(p, app.focus_layout);
    // let mut scroll_state: ScrollbarState = ScrollbarState::default().position(scroll_pos);
    // frame.render_stateful_widget(
    //     Scrollbar::new(ScrollbarOrientation::VerticalLeft)
    //         .symbols(scrollbar::VERTICAL)
    //         .begin_symbol(None)
    //         .track_symbol(None)
    //         .end_symbol(None),
    //     app.focus_layout.inner(Margin {
    //         vertical: 1,
    //         horizontal: 0,
    //     }),
    //     &mut scroll_state,
    // );
}

fn render_preview_pane(frame: &mut Frame, app: &App) {
    let width = app.preview_layout.width.into();
    let preview = match &app.app_cursor {
        Some(cursor) => {
            let selected = Path::new(&cursor.entry);
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
                    Text::from("Empty...").style(Style::default().fg(Color::Red).bg(Color::Black))
                } else {
                    Text::from(sub_paths)
                    // sub_paths.sort();
                    // sub_paths.join("\n")
                }
            } else if selected.is_file() {
                if selected
                    .extension()
                    .and_then(|e| e.to_str())
                    .map_or(false, |e| e.eq_ignore_ascii_case("pdf"))
                {
                    let doc = Document::load(selected);
                    match doc {
                        Ok(document) => {
                            let text = document
                                .extract_text(&[1])
                                .unwrap_or("Failed to parse PDF".to_string());
                            Text::from(text).style(Style::default().fg(Color::White))
                        }
                        Err(err) => Text::from(format!("{}", err)),
                    }
                } else {
                    Text::from(
                        fs::read_to_string(selected)
                            .unwrap_or_else(|_| "Failed to parse whatever that is".to_string()),
                    )
                    .style(Style::default().fg(Color::White))
                }
            } else {
                Text::from("Unkwnown entry type")
                    .style(Style::default().fg(Color::Red).bg(Color::Black))
            }
        }
        None => Text::from("nope"),
    };

    let p = Paragraph::new(preview)
        .block(Block::default())
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(p, app.preview_layout);
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
    if (matches!(ctx, PaneContext::Focus)
        && app.app_cursor.as_ref().map(|c| &c.entry).eq(&Some(&path)))
        || (matches!(ctx, PaneContext::Parent) && app.focus_dir.path == path)
        || (matches!(ctx, PaneContext::Preview) && (app.forward_stack.last() == Some(&path)))
    {
        bg_color = fg_color;
        fg_color = Color::Black;
    }

    let padded_name = format!("{:<width$}", basename);
    let style = Style::default().fg(fg_color).bg(bg_color);
    Line::from(padded_name).style(style)
}
