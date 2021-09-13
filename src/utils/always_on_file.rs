use super::super::configuration;
use super::AlwaysOn;

use std::convert::From;
use std::path::PathBuf;

#[derive(Debug)]
pub struct AlwaysOnFile {
    file: PathBuf,
}

impl AlwaysOnFile {
    #[allow(dead_code)]
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }
}

impl AlwaysOn for AlwaysOnFile {
    fn is_always_on(&self) -> bool {
        self.file.exists()
    }
}

impl From<&configuration::Files> for AlwaysOnFile {
    fn from(files: &configuration::Files) -> Self {
        Self::new(PathBuf::from(files.always_on.clone()))
    }
}
