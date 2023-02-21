#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait WakeupServer: Send + Sync {
    fn wakeup(&self) -> anyhow::Result<()>;
}
