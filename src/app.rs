use std::{
    collections::{HashMap, HashSet},
    env, error,
    fmt::Display,
    fs,
    path::PathBuf,
    process::Command,
};

use ratatui::layout::{Constraint, Direction, Layout, Rect, Size};

use crate::{directory_entry::DirectoryEntry, input::Input, tui::Tui};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[repr(u16)]
enum DefaultConstraints {
    Parent = 1,
    Focus = 2,
    Preview = 3,
}

#[derive(Debug)]
pub struct AppCursor {
    pub entry: PathBuf,
    pub idx: usize,
}

// impl Default for AppCursor {
//     fn default() -> Self {
//         Self {
//             entry: PathBuf::default(),
//             idx: 0,
//         }
//     }
// }

impl AppCursor {
    pub fn new(entry: PathBuf, idx: usize) -> Self {
        Self { entry, idx }
    }
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub needs_redraw: bool,

    area: Rect,

    pub titlebar_layout: Vec<Rect>,
    pub message: Option<String>,
    pub message_layout: Rect,

    parent_constraint: Constraint,
    parent_needs_reset: bool,
    focus_constraint: Constraint,
    preview_constraint: Constraint,
    explorer_layout: Vec<Rect>,
    base_layout: Vec<Rect>,

    pub parent_layout: Rect,
    pub focus_layout: Rect,
    pub preview_layout: Rect,

    pub show_preview: bool,

    pub focus_dir: DirectoryEntry,
    pub parent_dir: Option<DirectoryEntry>,
    pub preview: Option<PathBuf>,

    pub path_stack: Vec<PathBuf>,
    pub forward_stack: Vec<PathBuf>,
    pub selections: HashMap<PathBuf, HashSet<PathBuf>>, // TODO go back to hashMap so delete doesn't do bad things
    // pub cursor: Option<PathBuf>,
    // pub cursor_idx: usize,
    pub app_cursor: Option<AppCursor>,
    pub wrap: bool,

    pub input: Option<Input>,
}

impl Default for App {
    fn default() -> Self {
        let curr_path = env::current_dir().expect("Couldn't read path");
        let current_dir =
            DirectoryEntry::new(curr_path.clone()).expect("Problem when creating parent directory");

        let app_cursor = match current_dir.contents.get(0) {
            Some(entry) => {
                let entry_path = entry.to_path_buf();
                Some(AppCursor::new(entry_path, 0))
            }
            None => None,
        };

        Self {
            area: Rect::default(),
            show_preview: true,
            parent_constraint: Constraint::Fill(1),
            parent_needs_reset: false,
            focus_constraint: Constraint::Fill(2),
            // preview_constraint: Constraint::Percentage(DefaultConstraints::Preview as u16),
            preview_constraint: Constraint::Fill(3),
            running: true,
            needs_redraw: false,
            titlebar_layout: vec![Rect::default()],
            message_layout: Rect::default(),
            message: None,
            base_layout: vec![Rect::default()],
            explorer_layout: vec![Rect::default()],
            parent_layout: Rect::default(),
            focus_layout: Rect::default(),
            preview_layout: Rect::default(),

            parent_dir: match curr_path.parent() {
                Some(parent) => Some(
                    DirectoryEntry::new(parent.to_path_buf())
                        .expect("Problem when creating directory"),
                ),
                None => None,
            },
            path_stack: curr_path
                .ancestors()
                .skip(1) // skip current dir so path_stack.pop doesn't put me back in it
                .map(|a| a.to_path_buf())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
            forward_stack: Vec::new(),
            // cursor_idx: 0,
            focus_dir: current_dir,
            preview: app_cursor.as_ref().map(|c| c.entry.clone()),
            app_cursor,

            selections: HashMap::new(),
            wrap: true,
            input: None,
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(size: Size) -> Self {
        let mut app = Self::default();
        let area = Rect::new(0, 0, size.width, size.height);
        app.generate_layout(area);
        app.area = area;
        app
    }

    pub fn generate_layout(&mut self, area: Rect) {
        self.area = area;
        self.base_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Fill(1),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(area)
            .to_vec();

        self.titlebar_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Fill(2)].as_ref())
            .split(self.base_layout[0])
            .to_vec();

        self.explorer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    self.parent_constraint,
                    self.focus_constraint,
                    self.preview_constraint,
                ]
                .as_ref(),
            )
            .split(self.base_layout[1])
            .to_vec();

        self.parent_layout = self.explorer_layout[0];
        self.focus_layout = self.explorer_layout[1];
        self.preview_layout = self.explorer_layout[2];

        self.message_layout = self.base_layout[2];
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        let _ = match &mut self.parent_dir {
            Some(dir) => dir.update(),
            None => Ok(()),
        };

        let _ = self.focus_dir.update();
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    // pub fn update(&mut self) {
    //     let _ = match &mut self.parent_dir {
    //         Some(dir) => dir.update(),
    //         None => Ok(())
    //     };
    //     let _ = self.current_dir.update();
    // }

    pub fn move_up(&mut self) {
        if let Some(app_cursor) = &mut self.app_cursor {
            let length = self.focus_dir.contents.len();
            if app_cursor.idx >= length {
                app_cursor.idx = length - 1
            } else if app_cursor.idx > 0 {
                app_cursor.idx -= 1
            } else if self.wrap {
                app_cursor.idx = length - 1
            } else {
                return;
            }

            app_cursor.entry = self
                .focus_dir
                .contents
                .get(app_cursor.idx)
                .map(|path| path.to_path_buf())
                .expect(&format!("{}", &app_cursor.idx.to_string()));
        }
    }

    pub fn move_down(&mut self) {
        if let Some(app_cursor) = &mut self.app_cursor {
            if app_cursor.idx < self.focus_dir.contents.len() - 1 {
                app_cursor.idx += 1;
            } else if self.wrap {
                app_cursor.idx = 0
            } else {
                return;
            }

            app_cursor.entry = self
                .focus_dir
                .contents
                .get(app_cursor.idx)
                .map(|path| path.to_path_buf())
                .expect("why?");
        }
    }

    pub fn move_back(&mut self) {
        if let Some(path) = self.path_stack.pop() {
            let focus_dir_path = self.focus_dir.path.clone();

            self.focus_dir = DirectoryEntry::new(path).expect("Couldn't pop");

            let cursor_idx = match &self.parent_dir {
                Some(parent) => parent
                    .contents
                    .iter()
                    .position(|p| p == &focus_dir_path)
                    .unwrap_or(0),
                None => 0,
            };

            self.app_cursor = Some(AppCursor::new(focus_dir_path, cursor_idx));

            self.parent_dir = match self.focus_dir.path.parent() {
                Some(parent) => Some(
                    DirectoryEntry::new(parent.to_path_buf())
                        .expect("Problem when creating directory"),
                ),
                None => {
                    self.set_parent_constraint(0);
                    self.generate_layout(self.area);
                    None
                }
            };
        };
    }

    fn open_cursor(&mut self, entry: PathBuf) {
        // TODO handle results
        use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

        let _ = crossterm::execute!(std::io::stdout(), LeaveAlternateScreen);

        #[cfg(target_os = "macos")]
        Command::new("nvim")
            .arg(entry)
            .status()
            .expect("Failed to open nvim");

        #[cfg(target_os = "windows")] // TODO
        Command::new("explorer")
            .arg(".")
            .spawn()
            .expect("Failed to open directory");

        #[cfg(target_os = "linux")]
        Command::new("nvim")
            .arg(entry)
            .status()
            .expect("Failed to open nvim");

        let _ = crossterm::execute!(std::io::stdout(), EnterAlternateScreen);
    }

    pub fn move_into(&mut self) {
        match &mut self.app_cursor {
            Some(cursor) if cursor.entry.is_file() => {
                let entry = cursor.entry.clone();
                self.open_cursor(entry);
                // todo open in nvim
                return;
            }
            Some(cursor) => {
                let new_focus_dir = match DirectoryEntry::new(cursor.entry.clone()) {
                    Ok(dir) => dir,
                    Err(_) => {
                        // panic!("shit");
                        return;
                    }
                };

                let cursor_idx = if let Some(_) = self.forward_stack.pop() {
                    1
                } else {
                    0
                };
                let curr_dir = std::mem::take(&mut self.focus_dir);
                self.parent_dir = Some(curr_dir.clone());
                if self.parent_needs_reset {
                    self.reset_parent_constraint();
                    self.generate_layout(self.area);
                }
                self.path_stack.push(curr_dir.path);
                self.focus_dir = new_focus_dir;

                match self.focus_dir.contents.get(cursor_idx) {
                    Some(c) => {
                        let cursor_path = c.to_path_buf();
                        // let first_entry = cursor_path
                        //     .read_dir()
                        //     .map(|entry| entry.map(|p| p.expect("W").path()))
                        //     .expect(&format!("{}", cursor_path.display().to_string()))
                        //     .collect();

                        // self.forward_stack.push(first_entry);

                        // let mut contents = fs::read_dir(&path)?
                        //         .map(|res| res.map(|e| e.path()))
                        //         .collect::<Result<Vec<_>, io::Error>>()?;

                        self.app_cursor = Some(AppCursor::new(cursor_path, cursor_idx))
                    }
                    None => return,
                }
            }
            None => return,
        }
    }

    pub fn toggle_selection_on_cursor(&mut self) {
        let Some(cursor) = &self.app_cursor else {
            return;
        };

        let c = cursor.entry.clone();
        let selections = self.current_selections_mut();

        if !selections.remove(&c) {
            selections.insert(c);
        }
    }

    pub fn current_selections_mut(&mut self) -> &mut HashSet<PathBuf> {
        self.selections
            .entry(self.focus_dir.path.clone())
            .or_insert_with(HashSet::new)
    }

    pub fn current_selections(&self) -> HashSet<PathBuf> {
        self.selections
            .get(&self.focus_dir.path)
            .cloned()
            .unwrap_or_default()
    }

    pub fn delete_selection_or_cursor(&mut self) {
        // TODO some sort of trash bin to undo deletions?
        if !self.selections.is_empty() {
            if let Some(selections) = self.selections.get_mut(&self.focus_dir.path) {
                selections.retain(|path| {
                    match fs::remove_file(path).or_else(|_| fs::remove_dir_all(path)) {
                        Ok(_) => false,
                        Err(e) => {
                            eprintln!("{}", e);
                            true
                        }
                    }
                })
            }
        } else if let Some(cursor) = &self.app_cursor {
            let c_entry = cursor.entry.clone();

            let _ = if cursor.entry.is_file() {
                fs::remove_file(c_entry)
            } else {
                fs::remove_dir_all(c_entry)
            };
        }
    }

    pub fn set_parent_constraint(&mut self, percent: u16) {
        self.parent_needs_reset = true;
        // self.parent_constraint = Constraint::Percentage(percent);
        self.parent_constraint = Constraint::Fill(percent);
    }

    pub fn reset_parent_constraint(&mut self) {
        // self.parent_constraint = Constraint::Percentage(DefaultConstraints::Parent as u16);
        self.parent_constraint = Constraint::Fill(DefaultConstraints::Parent as u16);
    }

    // pub fn forward_path(&self) -> Option<&PathBuf> {
    //     self.forward_stack.last()
    // }

    pub fn show_deletion_msg(&mut self) {
        let selections_len = self
            .selections
            .get(&self.focus_dir.path)
            .map_or(0, |s| s.len());

        if selections_len != 0 {
            self.message = Some(format!(
                "Confirm deletion of {} files? [y/N]",
                selections_len
            ));
            return;
        }

        if let Some(cursor) = &self.app_cursor {
            self.message = Some(format!(
                "Confirm deletion of \"{}\" [y/N]",
                cursor
                    .entry
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ));
        }
    }

    pub fn show_rename_msg(&mut self) {
        // TODO batch rename in place, like oil.nvim
        //
        // maybe something like a Map of PathBuf to newName then loop over map
        // values and do fs::rename on them?
        //
        // the problem is that I need to map the "actions" in the sense that
        // multi-line editing should make the same changes to each line
        //
        // although... I could iterate through the selections* whenever a key
        // is pressed and call the same function on it (like insert_char etc)
        // and that would also allow for real-time visuals instead of what vim
        // does (updating all the lines only after ENTER)
        //
        // *I actually mean the focus_dir.contents because that's what's
        // displayed fr, but to avoid breaking shit maybe I should keep
        // two buffers? one with valid PathBufs and one with their display?
        // we'll see (who is we)

        // let selections_len = self
        //     .selections
        //     .get(&self.focus_dir.path)
        //     .map_or(0, |s| s.len());
        //
        // if selections_len != 0 {
        //     self.message = Some(format!(
        //         "Confirm deletion of {} files? [y/N]",
        //         selections_len
        //     ));
        //     return;
        // }

        if let Some(cursor) = &self.app_cursor {
            self.message = Some(format!(
                "Rename \"{}\" to: ",
                cursor
                    .entry
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ));

            self.input = Some(Input::default());
        }
    }

    pub fn clear_msg(&mut self) {
        self.message = None;
    }

    pub fn delete_char(&mut self) {
        if let Some(input) = &mut self.input {
            input.delete_char();
        }
    }

    pub fn terminate_input(&mut self) {
        if let Some(cursor) = &self.app_cursor {
            if let Some(input) = &self.input {
                let _ = fs::rename(&cursor.entry, &input.content);
                self.input = None;
            }
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        if let Some(input) = &mut self.input {
            input.insert_char(ch);
        }
    }
}
