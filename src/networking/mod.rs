use pnet::datalink::{interfaces, NetworkInterface};

pub mod controllable_server;
pub mod networking_error;
pub mod shutdown_error;
pub mod ssh2_server;

pub fn get_network_interface(
    interface_name: &str,
) -> Result<NetworkInterface, networking_error::NetworkingError> {
    // get all network interfaces
    let ifaces = interfaces();

    // try to find the interface matching the given name
    let iface = ifaces
        .into_iter()
        .find(|iface| iface.name == interface_name);
    return match iface {
        Some(iface) => Ok(iface),
        None => Err(networking_error::NetworkingError(format!(
            "unknown network interface: {}",
            interface_name
        ))),
    };
}
