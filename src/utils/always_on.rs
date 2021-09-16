#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait AlwaysOn: Send {
    fn is_always_on(&self) -> bool;
}
