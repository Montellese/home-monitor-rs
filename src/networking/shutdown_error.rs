use std::fmt;

#[derive(Debug)]
pub struct ShutdownError(String);

impl ShutdownError {
    pub fn new(error_msg: String) -> Self {
        Self(error_msg)
    }
}

impl std::error::Error for ShutdownError {}

impl fmt::Display for ShutdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ShutdownError] {}", self.0)
    }
}
