use super::configuration::machine::{Machine, Server};

use log::debug;

use pnet::datalink::{interfaces, NetworkInterface};

use std::fmt;

#[derive(Debug)]
pub struct NetworkingError(String);

impl fmt::Display for NetworkingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[NetworkingError] {}", self.0)
    }
}

pub struct Networking {
    pub interface: NetworkInterface,
}

impl Networking {
    pub fn create(interface_name: &str) -> Result<Networking, NetworkingError> {
        // get all network interfaces
        let ifaces = interfaces();

        // try to find the interface matching the given name
        let iface = ifaces.into_iter().find(|iface| iface.name == interface_name);
        match iface {
            Some(iface) => {
                Ok(Networking {
                    interface: iface,
                })
            },
            None => Err(NetworkingError(format!("unknown network interface: {}", interface_name)))
        }
    }
}

pub fn wakeup(machine: &Machine) -> std::io::Result<()> {
    debug!("sending wake-on-lan request to {} [{}]", machine.name, machine.mac);
    let wol = wakey::WolPacket::from_string(&machine.mac, ':');
    wol.send_magic()
}

pub fn shutdown(server: &Server) -> Result<bool, NetworkingError> {
    // TODO(Montellese)
    unimplemented!();
}