use super::super::dom::machine;

use super::wakeup_server::WakeupServer;

use log::debug;

pub struct WakeOnLanServer {
    name: String,
    mac: String,
}

impl WakeOnLanServer {
    pub fn new(server: &machine::Server) -> Self {
        WakeOnLanServer {
            name: server.machine.name.to_string(),
            mac: server.machine.mac.to_string(),
        }
    }
}

impl WakeupServer for WakeOnLanServer {
    fn wakeup(&self) -> std::io::Result<()> {
        debug!(
            "sending wake-on-lan request to {} [{}]",
            self.name, self.mac
        );
        let wol = wakey::WolPacket::from_string(&self.mac, ':');
        wol.send_magic()
    }
}
