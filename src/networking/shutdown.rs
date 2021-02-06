use super::super::configuration::machine::Server;

use log::debug;
use ssh2::Session;

use std::fmt;
use std::net::TcpStream;

#[derive(Debug)]
pub struct ShutdownError(String);

impl ShutdownError {
    pub fn new(error_msg: String) -> ShutdownError {
        ShutdownError(error_msg)
    }

    pub fn from_ssh2_error(e: ssh2::Error) -> ShutdownError {
        ShutdownError(format!("[{}] {}", e.code(), e.message()))
    }
}

impl std::error::Error for ShutdownError {}

impl fmt::Display for ShutdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ShutdownError] {}", self.0)
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