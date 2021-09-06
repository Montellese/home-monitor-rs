use std::fmt;

#[derive(Debug)]
pub struct NetworkingError(pub String);

impl fmt::Display for NetworkingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[NetworkingError] {}", self.0)
    }
}
