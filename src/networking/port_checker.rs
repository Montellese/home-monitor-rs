#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait PortChecker {
    fn check(&self) -> bool;
}
