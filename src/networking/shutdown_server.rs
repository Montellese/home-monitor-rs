use super::shutdown_error::ShutdownError;

pub trait ShutdownServer {
    fn shutdown(&self) -> Result<(), ShutdownError>;
}
