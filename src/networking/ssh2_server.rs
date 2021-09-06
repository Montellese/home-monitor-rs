use super::super::dom::machine;

use super::controllable_server::ControllableServer;
use super::shutdown_error::ShutdownError;

use log::debug;
use ssh2::Session;
use std::net::TcpStream;

pub struct Ssh2Server {
    name: String,
    mac: String,
    ip: String,
    username: String,
    password: String,
}

impl Ssh2Server {
    pub fn new(server: machine::Server) -> Self {
        Ssh2Server {
            name: server.machine.name.to_string(),
            mac: server.machine.mac.to_string(),
            ip: server.machine.ip.to_string(),
            username: server.username.to_string(),
            password: server.password.to_string(),
        }
    }

    fn ssh2_to_shutdown_error(e: ssh2::Error) -> ShutdownError {
        ShutdownError::new(format!("[{}] {}", e.code(), e.message()))
    }

    fn handle_shutdown_error<T>(result: Result<T, ssh2::Error>) -> Result<T, ShutdownError> {
        match result {
            Ok(r) => Ok(r),
            Err(e) => Err(Ssh2Server::ssh2_to_shutdown_error(e)),
        }
    }
}

impl ControllableServer for Ssh2Server {
    fn wakeup(&self) -> std::io::Result<()> {
        debug!(
            "sending wake-on-lan request to {} [{}]",
            self.name, self.mac
        );
        let wol = wakey::WolPacket::from_string(&self.mac, ':');
        wol.send_magic()
    }

    fn shutdown(&self) -> Result<(), ShutdownError> {
        debug!("creating an SSH session to {} [{}]", self.name, self.ip);
        let tcp = match TcpStream::connect(&self.ip) {
            Ok(s) => s,
            Err(e) => return Err(ShutdownError::new(format!("{}", e))),
        };
        let mut session = Ssh2Server::handle_shutdown_error(Session::new())?;
        session.set_tcp_stream(tcp);
        Ssh2Server::handle_shutdown_error(session.handshake())?;

        debug!(
            "authenticating SSH session to {} for {}",
            self.name, self.username
        );
        Ssh2Server::handle_shutdown_error(
            session.userauth_password(&self.username, &self.password),
        )?;

        debug!("executing \"shutdown -h now\" on {}", self.name);
        let mut channel = Ssh2Server::handle_shutdown_error(session.channel_session())?;
        Ssh2Server::handle_shutdown_error(channel.exec("shutdown -h now"))?;

        Ssh2Server::handle_shutdown_error(channel.wait_close())
    }
}
