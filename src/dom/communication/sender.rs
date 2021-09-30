#[cfg(test)]
use mockall::automock;

use super::Device;

#[cfg_attr(test, automock)]
pub trait Sender: Send {
    fn send(&self, device: Device) -> Result<(), Box<dyn std::error::Error>>;
}
