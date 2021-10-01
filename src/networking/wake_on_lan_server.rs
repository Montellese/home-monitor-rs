use log::debug;

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
    fn wakeup(&self) -> std::io::Result<()> {
        debug!(
            "sending wake-on-lan request to {} [{}]",
            self.name, self.mac
        );
        let wol = wakey::WolPacket::from_bytes(self.mac.as_bytes());
        wol.send_magic()
    }
}
