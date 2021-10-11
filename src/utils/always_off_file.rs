use std::convert::From;
use std::path::{Path, PathBuf};

use anyhow::anyhow;

use super::super::configuration;
use super::AlwaysOff;

#[derive(Debug)]
pub struct AlwaysOffFile {
    file: PathBuf,
}

impl AlwaysOffFile {
    #[allow(dead_code)]
    pub fn new(path: &Path) -> Self {
        // make sure the given path exists
        std::fs::create_dir_all(path).unwrap();

        // append alwaysoff to the path
        let mut file = path.to_path_buf();
        file.push("alwaysoff");

        Self { file }
    }

    #[cfg(test)]
    pub fn path(&self) -> &Path {
        &self.file
    }
}

impl AlwaysOff for AlwaysOffFile {
    fn is_always_off(&self) -> bool {
        self.file.exists()
    }

    fn set_always_off(&self) -> anyhow::Result<()> {
        std::fs::write(&self.file, "")?;
        Ok(())
    }

    fn reset_always_off(&self) -> anyhow::Result<()> {
        match std::fs::remove_file(&self.file) {
            Ok(_) => Ok(()),
            Err(e) => {
                use std::io::ErrorKind::*;
                match e.kind() {
                    // it's OK if the file didn't exist anyway
                    NotFound => Ok(()),
                    // otherwise return the error
                    _ => Err(anyhow!(e)),
                }
            }
        }
    }
}

impl From<&configuration::Files> for AlwaysOffFile {
    fn from(files: &configuration::Files) -> Self {
        Self::new(&files.root)
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use temp_dir::*;

    use super::*;

    #[fixture]
    fn root() -> TempDir {
        TempDir::new().unwrap()
    }

    fn create_file(always_off: &AlwaysOffFile) {
        assert!(std::fs::write(always_off.path(), "").is_ok());
    }

    #[rstest]
    fn is_always_off_fails_if_file_doesnt_exist(root: TempDir) {
        let always_off = AlwaysOffFile::new(root.path());

        assert!(!always_off.is_always_off());
    }

    #[rstest]
    fn is_always_off_succeeds_if_file_exists(root: TempDir) {
        let always_off = AlwaysOffFile::new(root.path());
        create_file(&always_off);

        assert!(always_off.is_always_off());
    }

    #[rstest]
    fn set_always_off_succeeds_if_file_doesnt_exist(root: TempDir) {
        let always_off = AlwaysOffFile::new(root.path());

        assert!(always_off.set_always_off().is_ok());
    }

    #[rstest]
    fn set_always_off_succeeds_if_file_exists(root: TempDir) {
        let always_off = AlwaysOffFile::new(root.path());
        create_file(&always_off);

        assert!(always_off.set_always_off().is_ok());
    }

    #[rstest]
    fn reset_always_off_succeeds_if_file_doesnt_exist(root: TempDir) {
        let always_off = AlwaysOffFile::new(root.path());

        assert!(always_off.reset_always_off().is_ok());
    }

    #[rstest]
    fn reset_always_off_succeeds_if_file_exists(root: TempDir) {
        let always_off = AlwaysOffFile::new(root.path());
        create_file(&always_off);

        assert!(always_off.reset_always_off().is_ok());
    }
}
