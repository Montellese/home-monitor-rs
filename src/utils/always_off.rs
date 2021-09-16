#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait AlwaysOff: Send {
    fn is_always_off(&self) -> bool;
}
