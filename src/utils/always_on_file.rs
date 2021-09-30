use std::convert::From;
use std::path::PathBuf;

use super::super::configuration;
use super::AlwaysOn;

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

    fn set_always_on(&self) -> Result<(), Box<dyn std::error::Error>> {
        match std::fs::write(&self.file, "") {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn reset_always_on(&self) -> Result<(), Box<dyn std::error::Error>> {
        match std::fs::remove_file(&self.file) {
            Ok(_) => Ok(()),
            Err(e) => {
                use std::io::ErrorKind::*;
                match e.kind() {
                    // it's OK if the file didn't exist anyway
                    NotFound => Ok(()),
                    // otherwise return the error
                    _ => Err(Box::new(e)),
                }
            }
        }
    }
}

impl From<&configuration::Files> for AlwaysOnFile {
    fn from(files: &configuration::Files) -> Self {
        Self::new(PathBuf::from(files.always_on.clone()))
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use tempfile::*;

    use super::*;

    fn get_path(file: &NamedTempFile) -> PathBuf {
        PathBuf::from(file.path())
    }

    #[fixture]
    fn file() -> NamedTempFile {
        NamedTempFile::new().unwrap()
    }

    #[rstest]
    fn is_always_on_fails_if_file_doesnt_exist(file: NamedTempFile) {
        let path = get_path(&file);
        file.close().unwrap();

        let always_on = AlwaysOnFile::new(path);
        assert!(!always_on.is_always_on());
    }

    #[rstest]
    fn is_always_on_succeeds_if_file_exists(file: NamedTempFile) {
        let path = get_path(&file);

        let always_on = AlwaysOnFile::new(path);
        assert!(always_on.is_always_on());
    }

    #[rstest]
    fn set_always_on_succeeds_if_file_doesnt_exist(file: NamedTempFile) {
        let path = get_path(&file);
        file.close().unwrap();

        let always_on = AlwaysOnFile::new(path);
        assert!(always_on.set_always_on().is_ok());
    }

    #[rstest]
    fn set_always_on_succeeds_if_file_exists(file: NamedTempFile) {
        let path = get_path(&file);

        let always_on = AlwaysOnFile::new(path);
        assert!(always_on.set_always_on().is_ok());
    }

    #[rstest]
    fn reset_always_on_succeeds_if_file_doesnt_exist(file: NamedTempFile) {
        let path = get_path(&file);
        file.close().unwrap();

        let always_on = AlwaysOnFile::new(path);
        assert!(always_on.reset_always_on().is_ok());
    }

    #[rstest]
    fn reset_always_on_succeeds_if_file_exists(file: NamedTempFile) {
        let path = get_path(&file);

        let always_on = AlwaysOnFile::new(path);
        assert!(always_on.reset_always_on().is_ok());
    }
}
