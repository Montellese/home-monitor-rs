use super::super::configuration;

use std::convert::From;
use std::fmt;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Machine {
    pub name: String,
    pub mac: String,
    pub ip: String,

    pub last_seen_timeout: u64,
    pub is_online: bool,
    pub last_seen: Option<Instant>,
}

impl Machine {
    #[allow(dead_code)]
    pub fn new(name: &str, mac: &str, ip: &str, last_seen_timeout: u64) -> Self {
        Self {
            name: name.to_string(),
            mac: mac.to_string(),
            ip: ip.to_string(),
            last_seen_timeout,
            is_online: false,
            last_seen: None,
        }
    }

    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
        if online {
            self.last_seen = Some(Instant::now());
        }
    }
}

impl From<&configuration::machine::Machine> for Machine {
    fn from(machine: &configuration::machine::Machine) -> Self {
        Self {
            name: machine.name.clone(),
            mac: machine.mac.clone(),
            ip: machine.ip.clone(),
            last_seen_timeout: machine.last_seen_timeout,
            is_online: false,
            last_seen: None,
        }
    }
}

impl fmt::Display for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}) ", self.name, self.ip)?;
        match self.last_seen {
            None => {
                write!(f, "🯄")
            }
            Some(_) => {
                if self.is_online {
                    write!(f, "↑")
                } else {
                    write!(f, "↓")
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Server {
    pub machine: Machine,

    pub username: String,
    pub password: String,
}

impl Server {
    #[allow(dead_code)]
    pub fn new(
        name: &str,
        mac: &str,
        ip: &str,
        last_seen_timeout: u64,
        username: &str,
        password: &str,
    ) -> Self {
        Self {
            machine: Machine::new(name, mac, ip, last_seen_timeout),
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}

impl From<&configuration::machine::Server> for Server {
    fn from(server: &configuration::machine::Server) -> Self {
        Self {
            machine: Machine::from(&server.machine),
            username: server.username.clone(),
            password: server.password.clone(),
        }
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.username, self.machine)
    }
}
