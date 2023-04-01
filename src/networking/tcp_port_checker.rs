use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

use log::debug;

use super::PortChecker;

pub struct TcpPortChecker {
    socket_addr: SocketAddr,
    timeout: Duration,
}

impl TcpPortChecker {
    pub fn new(ip: IpAddr, port: u16, timeout: Duration) -> Self {
        Self {
            socket_addr: SocketAddr::new(ip, port),
            timeout,
        }
    }
}

impl PortChecker for TcpPortChecker {
    fn check(&self) -> bool {
        debug!(
            "checking TCP port {} on {}",
            self.socket_addr.port(),
            self.socket_addr.ip()
        );
        TcpStream::connect_timeout(&self.socket_addr, self.timeout).is_ok()
    }
}
