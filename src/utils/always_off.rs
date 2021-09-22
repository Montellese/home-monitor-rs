#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait AlwaysOff: Send + Sync {
    fn is_always_off(&self) -> bool;
    fn set_always_off(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn reset_always_off(&self) -> Result<(), Box<dyn std::error::Error>>;
}
