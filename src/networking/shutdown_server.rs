#[cfg(test)]
use mockall::automock;

use super::ShutdownError;

#[cfg_attr(test, automock)]
pub trait ShutdownServer: Send + Sync {
    fn shutdown(&self) -> Result<(), ShutdownError>;
}
