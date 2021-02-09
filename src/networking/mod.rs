use super::configuration::machine::Machine;

use log::debug;
use pnet::datalink::{interfaces, NetworkInterface};

use std::fmt;

pub mod networking_error;
pub mod ping;
pub mod shutdown;

impl fmt::Display for networking_error::NetworkingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[NetworkingError] {}", self.0)
    }
}

pub fn get_network_interface(interface_name: &str) -> Result<NetworkInterface, networking_error::NetworkingError> {
    // get all network interfaces
    let ifaces = interfaces();

    // try to find the interface matching the given name
    let iface = ifaces.into_iter().find(|iface| iface.name == interface_name);
    return match iface {
        Some(iface) => Ok(iface),
        None => Err(networking_error::NetworkingError(format!("unknown network interface: {}", interface_name)))
    }
}

pub fn wakeup(machine: &Machine) -> std::io::Result<()> {
    debug!("sending wake-on-lan request to {} [{}]", machine.name, machine.mac);
    let wol = wakey::WolPacket::from_string(&machine.mac, ':');
    wol.send_magic()
}
