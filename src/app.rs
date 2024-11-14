use std::{
    collections::{HashMap, HashSet},
    env, error, fs,
    path::PathBuf,
};

use ratatui::layout::{Constraint, Direction, Layout, Rect, Size};

use crate::directory_entry::DirectoryEntry;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub titlebar_layout: Vec<Rect>,

    parent_constraint: Constraint,
    focus_constraint: Constraint,
    preview_constraint: Constraint,
    explorer_layout: Vec<Rect>,
    base_layout: Vec<Rect>,

    pub parent_layout: Rect,
    pub focus_layout: Rect,
    pub preview_layout: Rect,

    pub show_preview: bool,

    pub focus: DirectoryEntry,
    pub parent_dir: Option<DirectoryEntry>,
    pub preview: Option<PathBuf>,

    pub path_stack: Vec<(PathBuf, usize)>,
    pub selections: HashMap<PathBuf, HashSet<PathBuf>>, // TODO go back to hashMap so delete doesn't do bad things
    pub cursor: Option<PathBuf>,
    pub cursor_idx: usize,
    pub wrap: bool,
}

impl Default for App {
    fn default() -> Self {
        let curr_path = env::current_dir().expect("Couldn't read path");
        let current_dir =
            DirectoryEntry::new(curr_path.clone()).expect("Problem when creating parent directory");

        let cursor = match current_dir.contents.get(0) {
            Some(entry) => Some(entry.to_path_buf()),
            None => None,
        };

        Self {
            show_preview: true,
            parent_constraint: Constraint::Percentage(15),
            focus_constraint: Constraint::Percentage(35),
            preview_constraint: Constraint::Percentage(50),
            running: true,
            titlebar_layout: vec![Rect::default()],
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
                .map(|a| (a.to_path_buf(), 0))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
            cursor_idx: 0,
            focus: current_dir,
            preview: cursor.as_ref().cloned(),
            cursor,

            selections: HashMap::new(),
            wrap: true,
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
        self.base_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)].as_ref())
            .split(size)
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
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        let _ = match &mut self.parent_dir {
            Some(dir) => dir.update(),
            None => Ok(()),
        };

        let _ = self.focus.update();
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
        if self.cursor_idx > 0 {
            self.cursor_idx -= 1;
        } else if self.wrap {
            self.cursor_idx = self.focus.contents.len() - 1;
        } else {
            return;
        }

        self.cursor = self
            .focus
            .contents
            .get(self.cursor_idx)
            .map(|path| path.to_path_buf());
    }

    pub fn move_down(&mut self) {
        if self.cursor_idx < self.focus.contents.len() - 1 {
            self.cursor_idx += 1;
        } else if self.wrap {
            self.cursor_idx = 0;
        } else {
            return;
        }

        self.cursor = self
            .focus
            .contents
            .get(self.cursor_idx)
            .map(|path| path.to_path_buf());
    }

    pub fn move_back(&mut self) {
        if let Some((path, idx)) = self.path_stack.pop() {
            self.cursor = Some(self.focus.path.clone());
            self.cursor_idx = idx;
            self.focus = DirectoryEntry::new(path).expect("Couldn't pop");
            self.parent_dir = match self.focus.path.parent() {
                Some(parent) => Some(
                    DirectoryEntry::new(parent.to_path_buf())
                        .expect("Problem when creating directory"),
                ),
                None => None,
            };
        };
    }

    pub fn move_into(&mut self) {
        match &self.cursor {
            Some(cursor) => {
                if cursor.is_file() {
                    // todo open in nvim
                    return;
                }
            }
            None => return,
        }

        let selected_path = std::mem::take(&mut self.cursor).unwrap();
        let curr_dir = std::mem::take(&mut self.focus);
        self.parent_dir = Some(curr_dir.clone());
        self.path_stack.push((curr_dir.path, self.cursor_idx));
        self.focus = DirectoryEntry::new(selected_path).expect("Couldn't create directory");

        self.cursor_idx = 0;
        self.cursor = self
            .focus
            .contents
            .get(self.cursor_idx)
            .map(|path| path.to_path_buf());
    }

    pub fn toggle_selection_on_cursor(&mut self) {
        let cursor = self.cursor.clone();
        let selections = self.current_selections_mut();

        if let Some(c) = cursor {
            if !selections.remove(&c) {
                selections.insert(c);
            }
        }
    }

    pub fn current_selections_mut(&mut self) -> &mut HashSet<PathBuf> {
        self.selections
            .entry(self.focus.path.clone())
            .or_insert_with(HashSet::new)
    }

    pub fn current_selections(&self) -> HashSet<PathBuf> {
        self.selections
            .get(&self.focus.path)
            .cloned()
            .unwrap_or_default()
    }
}
