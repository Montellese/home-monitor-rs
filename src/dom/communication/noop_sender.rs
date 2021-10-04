use super::super::Device;
use super::Sender;

#[derive(Clone, Debug)]
pub struct NoopSender {}

impl NoopSender {
    pub fn new() -> Self {
        Self {}
    }
}

impl Sender for NoopSender {
    fn send(&self, _: Device) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
