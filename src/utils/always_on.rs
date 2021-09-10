#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait AlwaysOn {
    fn is_always_on(&self) -> bool;
}
