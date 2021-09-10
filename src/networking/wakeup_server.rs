#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait WakeupServer {
    fn wakeup(&self) -> std::io::Result<()>;
}
