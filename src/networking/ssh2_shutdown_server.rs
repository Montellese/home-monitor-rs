use std::net::TcpStream;
use std::path::Path;

use log::debug;
use ssh2::Session;

use super::super::dom;
use super::{ShutdownError, ShutdownServer};

struct PrivateKeyAuthentication {
    file: String,
    passphrase: String,
}

enum Authentication {
    Password(String),
    PrivateKey(PrivateKeyAuthentication),
}

pub struct Ssh2ShutdownServer {
    name: String,
    ip: String,
    username: String,
    authentication: Authentication,
}

impl Ssh2ShutdownServer {
    pub fn new(server: &dom::Server) -> Self {
        let authentication = match &server.authentication {
            dom::device::Authentication::Password(auth) => Authentication::Password(auth.clone()),
            dom::device::Authentication::PrivateKey(auth) => {
                Authentication::PrivateKey(PrivateKeyAuthentication {
                    file: auth.file.clone(),
                    passphrase: auth.passphrase.clone(),
                })
            }
        };

        Self {
            name: server.machine.name.to_string(),
            ip: server.machine.ip.to_string(),
            username: server.username.to_string(),
            authentication,
        }
    }

    fn ssh2_to_shutdown_error(e: ssh2::Error) -> ShutdownError {
        ShutdownError::new(format!(
            "[{code}] {message}",
            code = e.code(),
            message = e.message()
        ))
    }

    fn handle_shutdown_error<T>(result: Result<T, ssh2::Error>) -> Result<T, ShutdownError> {
        match result {
            Ok(r) => Ok(r),
            Err(e) => Err(Self::ssh2_to_shutdown_error(e)),
        }
    }

    fn connect(&self) -> Result<Session, ShutdownError> {
        debug!("creating an SSH session to {} [{}]", self.name, self.ip);
        let tcp = match TcpStream::connect(format!("{}:22", &self.ip)) {
            Ok(s) => s,
            Err(e) => return Err(ShutdownError::new(format!("{e}"))),
        };
        let mut session = Self::handle_shutdown_error(Session::new())?;
        session.set_tcp_stream(tcp);
        Self::handle_shutdown_error(session.handshake())?;

        self.authenticate(&session)?;

        Ok(session)
    }

    fn authenticate(&self, session: &Session) -> Result<(), ShutdownError> {
        match &self.authentication {
            Authentication::Password(password) => {
                debug!(
                    "authenticating SSH session to {} for {} using password",
                    self.name, self.username
                );
                Self::handle_shutdown_error(session.userauth_password(&self.username, password))?;
            }
            Authentication::PrivateKey(pk) => {
                debug!(
                    "authenticating SSH session to {} for {} using private key",
                    self.name, self.username
                );

                // make sure the private key exists
                let pk_path = Path::new(&pk.file);
                match pk_path.try_exists() {
                    Ok(exists) => {
                        if !exists {
                            return Err(ShutdownError::new(
                                format!("missing private key at {} to authenticate SSH session to {} for {}",
                                    pk.file, self.name, self.username)));
                        }
                    },
                    Err(err) => return Err(ShutdownError::new(
                        format!("error loading private key from {}to authenticate SSH session to {} for {}: {}",
                            pk.file, self.name, self.username, err))),
                }

                Self::handle_shutdown_error(session.userauth_pubkey_file(
                    &self.username,
                    Option::None,
                    pk_path,
                    Some(&pk.passphrase),
                ))?;
            }
        }

        Ok(())
    }
}

impl ShutdownServer for Ssh2ShutdownServer {
    fn shutdown(&self) -> Result<(), ShutdownError> {
        let session = self.connect()?;

        debug!("executing \"shutdown -h now\" on {}", self.name);
        let mut channel = Self::handle_shutdown_error(session.channel_session())?;
        Self::handle_shutdown_error(channel.exec("shutdown -h now"))?;

        Self::handle_shutdown_error(channel.wait_close())
    }
}
