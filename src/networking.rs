use super::configuration::machine::{Machine, Server};

use log::debug;
use pnet::datalink::{interfaces, NetworkInterface};
use ssh2::Session;

use std::fmt;
use std::net::TcpStream;

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

#[derive(Debug)]
pub struct ShutdownError {
    msg: String,
}

impl ShutdownError {
    pub fn new(error_msg: String) -> ShutdownError {
        ShutdownError {
            msg: error_msg,
        }
    }

    pub fn from_ssh2_error(e: ssh2::Error) -> ShutdownError {
        ShutdownError {
            msg: format!("[{}] {}", e.code(), e.message()),
        }
    }
}

impl std::error::Error for ShutdownError {}

impl fmt::Display for ShutdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "shutdown failed: {}", self.msg)
    }
}

fn handle_shutdown_error<T>(result: Result<T, ssh2::Error>) -> Result<T, ShutdownError> {
    match result {
        Ok(r) => Ok(r),
        Err(e) => Err(ShutdownError::from_ssh2_error(e)),
    }
}

pub fn shutdown(server: &Server) -> Result<(), ShutdownError> {
    debug!("creating an SSH session to {} [{}]", server.machine.name, server.machine.ip);
    let tcp = match TcpStream::connect(&server.machine.ip) {
        Ok(s) => s,
        Err(e) => return Err(ShutdownError::new(format!("{}", e))),
    };
    let mut session = handle_shutdown_error(Session::new())?;
    session.set_tcp_stream(tcp);
    handle_shutdown_error(session.handshake())?;

    debug!("authenticating SSH session to {} for {}", server.machine.name, server.username);
    handle_shutdown_error(session.userauth_password(&server.username, &server.password))?;

    debug!("executing \"shutdown -h now\" on {}", server.machine.name);
    let mut channel = handle_shutdown_error(session.channel_session())?;
    handle_shutdown_error(channel.exec("shutdown -h now"))?;

    handle_shutdown_error(channel.wait_close())
}