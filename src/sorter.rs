use std::{cmp::Ordering, fs::DirEntry, path::PathBuf};

pub enum Sorter {
    DirsFirst,
    FilesFirst,
    Alphabetical,
}

impl Sorter {
    pub fn paths(&self, a: &PathBuf, b: &PathBuf) -> Ordering {
        match self {
            Sorter::DirsFirst => match (a.is_dir(), b.is_dir()) {
                // using just cmp on file_names compares lexographically
                // meaning that uppercase files would come before lowercase
                (true, true) | (false, false) => {
                    a.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_lowercase()
                        .cmp(&b.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_lowercase())
                }
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
            },
            Sorter::FilesFirst => match (a.is_dir(), b.is_dir()) {
                (true, true) | (false, false) => a.file_name().cmp(&b.file_name()),
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
            },
            Sorter::Alphabetical => a.file_name().cmp(&b.file_name()),
        }
    }

    pub fn entries(&self, a: &DirEntry, b: &DirEntry) -> Ordering {
        match self {
            Sorter::DirsFirst => {
                let a_is_dir = a.path().is_dir();
                let b_is_dir = b.path().is_dir();

                match (a_is_dir, b_is_dir) {
                (true, true) | (false, false) => {
                    a.file_name()
                        .to_string_lossy()
                        .to_lowercase()
                        .cmp(&b.file_name()
                            .to_string_lossy()
                            .to_lowercase())
                }
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                }
            }
            Sorter::FilesFirst => {
                let a_is_dir = a.path().is_dir();
                let b_is_dir = b.path().is_dir();

                match (a_is_dir, b_is_dir) {
                    (true, true) | (false, false) => a.file_name().cmp(&b.file_name()),
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                }
            }
            Sorter::Alphabetical => a.file_name().cmp(&b.file_name()),
        }
    }
}
