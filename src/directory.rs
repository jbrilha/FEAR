use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct Directory {
    pub path: PathBuf,
    pub parent: Option<PathBuf>,
    pub contents: Vec<PathBuf>,
    pub cursor_idx: usize,
    last_update: Instant,
}

impl Default for Directory {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            parent: None,
            contents: Vec::new(),
            cursor_idx: usize::default(),
            last_update: Instant::now(),
        }
    }
}

impl Directory {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let mut contents = fs::read_dir(&path)?
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, io::Error>>()?;
        contents.sort();

        Ok(Self {
            contents,
            parent: path.parent().map(Path::to_path_buf),
            path,
            cursor_idx: 0,
            last_update: Instant::now(),
        })
    }

    fn should_update(&self) -> bool {
        self.last_update.elapsed() >= Duration::from_secs(1)
    }

    pub fn update(&mut self) -> io::Result<()> {
        if !self.should_update() { return Ok(()); }
            
        self.contents = fs::read_dir(&self.path)?
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, io::Error>>()?;
        self.contents.sort();

        self.last_update = Instant::now();
        Ok(())
    }
}
