use std::convert::From;
use std::path::{Path, PathBuf};

use anyhow::anyhow;

use super::super::configuration;
use super::AlwaysOn;

#[derive(Debug)]
pub struct AlwaysOnFile {
    file: PathBuf,
}

impl AlwaysOnFile {
    #[allow(dead_code)]
    pub fn new(path: &Path) -> Self {
        // make sure the given path exists
        std::fs::create_dir_all(path).unwrap();

        // append alwayson to the path
        let mut file = path.to_path_buf();
        file.push("alwayson");

        Self { file }
    }

    #[cfg(test)]
    pub fn path(&self) -> &Path {
        &self.file
    }
}

impl AlwaysOn for AlwaysOnFile {
    fn is_always_on(&self) -> bool {
        self.file.exists()
    }

    fn set_always_on(&self) -> anyhow::Result<()> {
        std::fs::write(&self.file, "")?;
        Ok(())
    }

    fn reset_always_on(&self) -> anyhow::Result<()> {
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

impl From<&configuration::Files> for AlwaysOnFile {
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

    fn create_file(always_on: &AlwaysOnFile) {
        assert!(std::fs::write(always_on.path(), "").is_ok());
    }

    #[rstest]
    fn is_always_on_fails_if_file_doesnt_exist(root: TempDir) {
        let always_on = AlwaysOnFile::new(root.path());

        assert!(!always_on.is_always_on());
    }

    #[rstest]
    fn is_always_on_succeeds_if_file_exists(root: TempDir) {
        let always_on = AlwaysOnFile::new(root.path());
        create_file(&always_on);

        assert!(always_on.is_always_on());
    }

    #[rstest]
    fn set_always_on_succeeds_if_file_doesnt_exist(root: TempDir) {
        let always_on = AlwaysOnFile::new(root.path());
        assert!(always_on.set_always_on().is_ok());
    }

    #[rstest]
    fn set_always_on_succeeds_if_file_exists(root: TempDir) {
        let always_on = AlwaysOnFile::new(root.path());
        create_file(&always_on);

        assert!(always_on.set_always_on().is_ok());
    }

    #[rstest]
    fn reset_always_on_succeeds_if_file_doesnt_exist(root: TempDir) {
        let always_on = AlwaysOnFile::new(root.path());
        assert!(always_on.reset_always_on().is_ok());
    }

    #[rstest]
    fn reset_always_on_succeeds_if_file_exists(root: TempDir) {
        let always_on = AlwaysOnFile::new(root.path());
        create_file(&always_on);

        assert!(always_on.reset_always_on().is_ok());
    }
}
