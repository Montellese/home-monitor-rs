use std::convert::From;
use std::path::PathBuf;

use super::super::configuration;
use super::AlwaysOff;

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

    fn set_always_off(&self) -> Result<(), Box<dyn std::error::Error>> {
        match std::fs::write(&self.file, "") {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn reset_always_off(&self) -> Result<(), Box<dyn std::error::Error>> {
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

impl From<&configuration::Files> for AlwaysOffFile {
    fn from(files: &configuration::Files) -> Self {
        Self::new(PathBuf::from(files.always_off.clone()))
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
    fn is_always_off_fails_if_file_doesnt_exist(file: NamedTempFile) {
        let path = get_path(&file);
        file.close().unwrap();

        let always_off = AlwaysOffFile::new(path);
        assert!(!always_off.is_always_off());
    }

    #[rstest]
    fn is_always_off_succeeds_if_file_exists(file: NamedTempFile) {
        let path = get_path(&file);

        let always_off = AlwaysOffFile::new(path);
        assert!(always_off.is_always_off());
    }

    #[rstest]
    fn set_always_off_succeeds_if_file_doesnt_exist(file: NamedTempFile) {
        let path = get_path(&file);
        file.close().unwrap();

        let always_off = AlwaysOffFile::new(path);
        assert!(always_off.set_always_off().is_ok());
    }

    #[rstest]
    fn set_always_off_succeeds_if_file_exists(file: NamedTempFile) {
        let path = get_path(&file);

        let always_off = AlwaysOffFile::new(path);
        assert!(always_off.set_always_off().is_ok());
    }

    #[rstest]
    fn reset_always_off_succeeds_if_file_doesnt_exist(file: NamedTempFile) {
        let path = get_path(&file);
        file.close().unwrap();

        let always_off = AlwaysOffFile::new(path);
        assert!(always_off.reset_always_off().is_ok());
    }

    #[rstest]
    fn reset_always_off_succeeds_if_file_exists(file: NamedTempFile) {
        let path = get_path(&file);

        let always_off = AlwaysOffFile::new(path);
        assert!(always_off.reset_always_off().is_ok());
    }
}
