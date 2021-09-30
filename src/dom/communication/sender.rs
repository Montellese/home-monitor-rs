use super::Device;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Sender: Send {
    fn send(&self, device: Device) -> Result<(), Box<dyn std::error::Error>>;
}
