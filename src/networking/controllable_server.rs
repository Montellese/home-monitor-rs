use super::shutdown_error::ShutdownError;

pub trait ControllableServer {
    fn wakeup(&self) -> std::io::Result<()>;
    fn shutdown(&self) -> Result<(), ShutdownError>;
}
