use std::{
    collections::{HashMap, HashSet},
    env, error,
    path::PathBuf,
};

use ratatui::layout::{Constraint, Direction, Layout, Rect, Size};

use crate::directory::Directory;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    // pub counter: u8,
    pub titlebar_layout: Vec<Rect>,
    pub explorer_layout: Vec<Rect>,
    pub base_layout: Vec<Rect>,

    pub current_dir: Directory,
    pub parent_dir: Option<Directory>,

    pub path_stack: Vec<(PathBuf, usize)>,
    pub selections: HashMap<PathBuf, HashSet<PathBuf>>,
    pub cursor: PathBuf,
    pub cursor_idx: usize,
    pub wrap: bool,
}

impl Default for App {
    fn default() -> Self {
        let curr_path = env::current_dir().expect("Couldn't read path");
        // let path_stack = vec![curr_path.clone()];
        let current_dir =
            Directory::new(curr_path.clone()).expect("Problem when creating parent directory");

        Self {
            running: true,
            titlebar_layout: vec![Rect::default()],
            base_layout: vec![Rect::default()],
            explorer_layout: vec![Rect::default()],

            parent_dir: match curr_path.parent() {
                Some(parent) => Some(
                    Directory::new(parent.to_path_buf()).expect("Problem when creating directory"),
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
            cursor: current_dir.contents.get(0).expect("Oh shit").to_path_buf(),
            current_dir,

            selections: HashMap::from_iter([(curr_path, HashSet::new())]),
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
            .constraints([Constraint::Percentage(10), Constraint::Fill(1)].as_ref())
            .split(size)
            .to_vec();

        self.titlebar_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1)].as_ref())
            .split(self.base_layout[0])
            .to_vec();

        self.explorer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(15),
                    Constraint::Percentage(35),
                    Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(self.base_layout[1])
            .to_vec();
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn move_up(&mut self) {
        if self.cursor_idx > 0 {
            self.cursor_idx -= 1;
        } else if self.wrap {
            self.cursor_idx = self.current_dir.contents.len() - 1;
        } else {
            return;
        }

        self.cursor = self
            .current_dir
            .contents
            .get(self.cursor_idx)
            .expect("Yo!")
            .to_path_buf();
    }

    pub fn move_down(&mut self) {
        if self.cursor_idx < self.current_dir.contents.len() - 1 {
            self.cursor_idx += 1;
        } else if self.wrap {
            self.cursor_idx = 0;
        } else {
            return;
        }

        self.cursor = self
            .current_dir
            .contents
            .get(self.cursor_idx)
            .expect("Yo!")
            .to_path_buf();
    }

    pub fn move_back(&mut self) {
        if let Some((path, idx)) = self.path_stack.pop() {
            self.cursor = self.current_dir.path.clone();
            self.cursor_idx = idx;
            self.current_dir = Directory::new(path).expect("Couldn't pop");
            self.parent_dir = match self.current_dir.path.parent() {
                Some(parent) => Some(
                    Directory::new(parent.to_path_buf()).expect("Problem when creating directory"),
                ),
                None => None,
            };
        };
    }

    pub fn move_into(&mut self) {
        if self.cursor.is_file() {
            // todo open in nvim
            return;
        }

        let selected_path = std::mem::take(&mut self.cursor);
        let curr_dir = std::mem::take(&mut self.current_dir);
        self.parent_dir = Some(curr_dir.clone());
        self.path_stack.push((curr_dir.path, self.cursor_idx));
        self.current_dir = Directory::new(selected_path).expect("Couldn't create directory");

        self.cursor_idx = 0;
        self.cursor = self
            .current_dir
            .contents
            .get(self.cursor_idx)
            .expect("Oh shit!!")
            .to_path_buf();
    }

    pub fn toggle_selection_on_cursor(&mut self) {
        let cursor = self.cursor.clone();
        let selections = self.current_selections_mut();

        if !selections.remove(&cursor) {
            selections.insert(cursor);
        }
    }

    pub fn current_selections_mut(&mut self) -> &mut HashSet<PathBuf> {
        self.selections
            .entry(self.current_dir.path.clone())
            .or_insert_with(HashSet::new)
    }

    pub fn current_selections(&self) -> HashSet<PathBuf> {
        self.selections
            .get(&self.current_dir.path)
            .cloned()
            .unwrap_or_default()
    }
}
