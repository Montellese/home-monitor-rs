use super::super::dom;

use super::ShutdownError;
use super::ShutdownServer;

use log::debug;
use ssh2::Session;
use std::net::TcpStream;

pub struct Ssh2ShutdownServer {
    name: String,
    ip: String,
    username: String,
    password: String,
}

impl Ssh2ShutdownServer {
    pub fn new(server: &dom::Server) -> Self {
        Self {
            name: server.machine.name.to_string(),
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
            Err(e) => Err(Self::ssh2_to_shutdown_error(e)),
        }
    }
}

impl ShutdownServer for Ssh2ShutdownServer {
    fn shutdown(&self) -> Result<(), ShutdownError> {
        debug!("creating an SSH session to {} [{}]", self.name, self.ip);
        let tcp = match TcpStream::connect(&self.ip) {
            Ok(s) => s,
            Err(e) => return Err(ShutdownError::new(format!("{}", e))),
        };
        let mut session = Self::handle_shutdown_error(Session::new())?;
        session.set_tcp_stream(tcp);
        Self::handle_shutdown_error(session.handshake())?;

        debug!(
            "authenticating SSH session to {} for {}",
            self.name, self.username
        );
        Self::handle_shutdown_error(session.userauth_password(&self.username, &self.password))?;

        debug!("executing \"shutdown -h now\" on {}", self.name);
        let mut channel = Self::handle_shutdown_error(session.channel_session())?;
        Self::handle_shutdown_error(channel.exec("shutdown -h now"))?;

        Self::handle_shutdown_error(channel.wait_close())
    }
}
