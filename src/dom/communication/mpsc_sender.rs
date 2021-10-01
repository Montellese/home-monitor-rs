use tokio::sync::mpsc;

use super::super::Device;
use super::Sender;

#[derive(Clone, Debug)]
pub struct MpscSender {
    sender: mpsc::UnboundedSender<Device>,
}

impl MpscSender {
    pub fn new(sender: mpsc::UnboundedSender<Device>) -> Self {
        Self { sender }
    }
}

impl Sender for MpscSender {
    fn send(&self, device: Device) -> Result<(), Box<dyn std::error::Error>> {
        match self.sender.send(device) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}
