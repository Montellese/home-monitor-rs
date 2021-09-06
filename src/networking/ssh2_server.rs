use super::super::configuration;

use super::controllable_server::ControllableServer;
use super::shutdown_error::ShutdownError;

use log::debug;
use ssh2::Session;
use std::net::TcpStream;

pub struct Ssh2Server {
    server: configuration::machine::Server,
}

impl Ssh2Server {
    pub fn new(server: configuration::machine::Server) -> Self {
        Ssh2Server { server }
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
        let machine = &self.server.machine;

        debug!(
            "sending wake-on-lan request to {} [{}]",
            machine.name, machine.mac
        );
        let wol = wakey::WolPacket::from_string(&machine.mac, ':');
        wol.send_magic()
    }

    fn shutdown(&self) -> Result<(), ShutdownError> {
        let server = &self.server;

        debug!(
            "creating an SSH session to {} [{}]",
            server.machine.name, server.machine.ip
        );
        let tcp = match TcpStream::connect(&server.machine.ip) {
            Ok(s) => s,
            Err(e) => return Err(ShutdownError::new(format!("{}", e))),
        };
        let mut session = Ssh2Server::handle_shutdown_error(Session::new())?;
        session.set_tcp_stream(tcp);
        Ssh2Server::handle_shutdown_error(session.handshake())?;

        debug!(
            "authenticating SSH session to {} for {}",
            server.machine.name, server.username
        );
        Ssh2Server::handle_shutdown_error(
            session.userauth_password(&server.username, &server.password),
        )?;

        debug!("executing \"shutdown -h now\" on {}", server.machine.name);
        let mut channel = Ssh2Server::handle_shutdown_error(session.channel_session())?;
        Ssh2Server::handle_shutdown_error(channel.exec("shutdown -h now"))?;

        Ssh2Server::handle_shutdown_error(channel.wait_close())
    }
}
