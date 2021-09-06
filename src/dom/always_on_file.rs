use super::super::configuration::files;
use super::always_on::AlwaysOn;

use std::path::PathBuf;

#[derive(Debug)]
pub struct AlwaysOnFile {
    file: PathBuf,
}

impl AlwaysOnFile {
    pub fn new(files: &files::Files) -> Self {
        AlwaysOnFile {
            file: PathBuf::from(files.always_on.clone()),
        }
    }
}

impl AlwaysOn for AlwaysOnFile {
    fn is_always_on(&self) -> bool {
        self.file.exists()
    }
}
