#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait AlwaysOn: Send {
    fn is_always_on(&self) -> bool;
    fn set_always_on(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn reset_always_on(&self) -> Result<(), Box<dyn std::error::Error>>;
}
