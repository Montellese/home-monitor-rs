use super::super::configuration;
use super::AlwaysOff;

use std::convert::From;
use std::path::PathBuf;

#[derive(Debug)]
pub struct AlwaysOffFile {
    file: PathBuf,
}

impl AlwaysOffFile {
    #[allow(dead_code)]
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }
}

impl AlwaysOff for AlwaysOffFile {
    fn is_always_off(&self) -> bool {
        self.file.exists()
    }
}

impl From<&configuration::Files> for AlwaysOffFile {
    fn from(files: &configuration::Files) -> Self {
        Self::new(PathBuf::from(files.always_off.clone()))
    }
}
