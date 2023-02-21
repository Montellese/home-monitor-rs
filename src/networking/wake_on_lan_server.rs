use anyhow::anyhow;
use log::{debug, warn};

use super::super::dom;
use super::super::utils::MacAddr;
use super::WakeupServer;

pub struct WakeOnLanServer {
    name: String,
    mac: MacAddr,
}

impl WakeOnLanServer {
    pub fn new(server: &dom::Server) -> Self {
        Self {
            name: server.machine.name.to_string(),
            mac: server.mac,
        }
    }
}

impl WakeupServer for WakeOnLanServer {
    fn wakeup(&self) -> anyhow::Result<()> {
        debug!(
            "sending wake-on-lan request to {} [{}]",
            self.name, self.mac
        );
        let wol = match wakey::WolPacket::from_bytes(self.mac.as_bytes()) {
            Err(e) => {
                warn!(
                    "failed to create wake-on-lan packet for {} [{}]: {}",
                    self.name, self.mac, e
                );
                Err(anyhow!(e))
            }
            Ok(wol) => Ok(wol),
        };

        match wol?.send_magic() {
            Err(e) => {
                warn!(
                    "failed to send wake-on-lan packet {} [{}]: {}",
                    self.name, self.mac, e
                );
                Err(anyhow!(e))
            }
            Ok(_) => Ok(()),
        }
    }
}
