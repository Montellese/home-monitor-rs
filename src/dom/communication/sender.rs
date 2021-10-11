#[cfg(test)]
use mockall::automock;

use super::super::Device;

#[cfg_attr(test, automock)]
pub trait Sender: Send {
    fn send(&self, device: Device) -> anyhow::Result<()>;
}
