use super::ShutdownError;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait ShutdownServer: Send + Sync {
    fn shutdown(&self) -> Result<(), ShutdownError>;
}
