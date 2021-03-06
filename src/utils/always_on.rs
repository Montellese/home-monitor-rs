#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait AlwaysOn: Send + Sync {
    fn is_always_on(&self) -> bool;
    fn set_always_on(&self) -> anyhow::Result<()>;
    fn reset_always_on(&self) -> anyhow::Result<()>;
}
