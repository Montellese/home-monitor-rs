use super::shutdown_error::ShutdownError;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait ShutdownServer {
    fn shutdown(&self) -> Result<(), ShutdownError>;
}
