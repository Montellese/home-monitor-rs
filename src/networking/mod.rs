use pnet::datalink::{interfaces, NetworkInterface};

mod fast_pinger;
mod networking_error;
mod pinger;
mod shutdown_error;
mod shutdown_server;
mod ssh2_shutdown_server;
mod wake_on_lan_server;
mod wakeup_server;

pub use fast_pinger::FastPinger;
pub use networking_error::NetworkingError;
#[cfg(test)]
pub use pinger::MockPinger;
pub use pinger::Pinger;
pub use shutdown_error::ShutdownError;
#[cfg(test)]
pub use shutdown_server::MockShutdownServer;
pub use shutdown_server::ShutdownServer;
pub use ssh2_shutdown_server::Ssh2ShutdownServer;
pub use wake_on_lan_server::WakeOnLanServer;
#[cfg(test)]
pub use wakeup_server::MockWakeupServer;
pub use wakeup_server::WakeupServer;

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
