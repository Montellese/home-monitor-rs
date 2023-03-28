use std::convert::From;
use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;
use std::string::ToString;

use chrono::{offset, DateTime, Utc};

use super::super::configuration;
use super::super::utils::{Instant, MacAddr};

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeviceId(pub String);

impl FromStr for DeviceId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl From<&configuration::DeviceId> for DeviceId {
    fn from(device_id: &configuration::DeviceId) -> Self {
        DeviceId(device_id.0.clone())
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Machine {
    pub id: DeviceId,
    pub name: String,
    pub ip: IpAddr,

    pub last_seen_timeout: u64,
    pub is_online: bool,
    pub last_seen: Option<Instant>,
    pub last_seen_date: Option<DateTime<Utc>>,
}

impl Machine {
    #[allow(dead_code)]
    pub fn new(id: &DeviceId, name: &str, ip: IpAddr, last_seen_timeout: u64) -> Self {
        Self {
            id: id.clone(),
            name: name.to_string(),
            ip,
            last_seen_timeout,
            is_online: false,
            last_seen: None,
            last_seen_date: None,
        }
    }

    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
        if online {
            self.last_seen = Some(Instant::now());
            self.last_seen_date = Some(offset::Utc::now());
        }
    }
}

impl From<&configuration::Machine> for Machine {
    fn from(machine: &configuration::Machine) -> Self {
        Self::new(
            &DeviceId::from(&machine.id),
            &machine.name,
            machine.ip,
            machine.last_seen_timeout,
        )
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivateKeyAuthentication {
    pub file: String,
    pub passphrase: String,
}

impl From<&configuration::PrivateKeyAuthentication> for PrivateKeyAuthentication {
    fn from(pk_auth: &configuration::PrivateKeyAuthentication) -> Self {
        Self {
            file: pk_auth.file.clone(),
            passphrase: pk_auth.passphrase.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Authentication {
    Password(String),
    PrivateKey(PrivateKeyAuthentication),
}

impl From<&configuration::Authentication> for Authentication {
    fn from(auth: &configuration::Authentication) -> Self {
        match auth {
            configuration::Authentication::Password(password_auth) => {
                Authentication::Password(password_auth.clone())
            }
            configuration::Authentication::PrivateKey(pk_auth) => {
                Authentication::PrivateKey(PrivateKeyAuthentication::from(pk_auth))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Server {
    pub machine: Machine,

    pub mac: MacAddr,
    pub username: String,
    pub authentication: Authentication,
}

impl Server {
    #[allow(dead_code)]
    pub fn new(
        id: &DeviceId,
        name: &str,
        ip: IpAddr,
        last_seen_timeout: u64,
        mac: MacAddr,
        username: &str,
        authentication: Authentication,
    ) -> Self {
        Self {
            machine: Machine::new(id, name, ip, last_seen_timeout),
            mac,
            username: username.to_string(),
            authentication,
        }
    }
}

impl From<&configuration::Server> for Server {
    fn from(server: &configuration::Server) -> Self {
        Self {
            machine: Machine::from(&server.machine),
            mac: server.mac,
            username: server.username.clone(),
            authentication: Authentication::from(&server.authentication),
        }
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.username, self.machine)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Device {
    Server(Server),
    Machine(Machine),
}

impl Device {
    #[allow(dead_code)]
    pub fn id(&self) -> &DeviceId {
        match self {
            Device::Server(server) => &server.machine.id,
            Device::Machine(machine) => &machine.id,
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &String {
        match self {
            Device::Server(server) => &server.machine.name,
            Device::Machine(machine) => &machine.name,
        }
    }

    #[allow(dead_code)]
    pub fn ip(&self) -> &IpAddr {
        match self {
            Device::Server(server) => &server.machine.ip,
            Device::Machine(machine) => &machine.ip,
        }
    }

    #[allow(dead_code)]
    pub fn last_seen_timeout(&self) -> u64 {
        match self {
            Device::Server(server) => server.machine.last_seen_timeout,
            Device::Machine(machine) => machine.last_seen_timeout,
        }
    }

    #[allow(dead_code)]
    pub fn last_seen(&self) -> Option<Instant> {
        match self {
            Device::Server(server) => server.machine.last_seen,
            Device::Machine(machine) => machine.last_seen,
        }
    }

    #[allow(dead_code)]
    pub fn is_online(&self) -> bool {
        match self {
            Device::Server(server) => server.machine.is_online,
            Device::Machine(machine) => machine.is_online,
        }
    }

    #[allow(dead_code)]
    pub fn set_online(&mut self, online: bool) {
        match self {
            Device::Server(server) => server.machine.set_online(online),
            Device::Machine(machine) => machine.set_online(online),
        };
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Device::Server(server) => fmt::Display::fmt(server, f),
            Device::Machine(machine) => fmt::Display::fmt(machine, f),
        }
    }
}

#[cfg(test)]
pub mod test {
    use rstest::*;

    use super::*;

    pub static SERVER_ID: &str = "testserver";
    pub static SERVER_NAME: &str = "Test Server";
    pub static SERVER_MAC: &str = "aa:bb:cc:dd:ee:ff";
    pub static SERVER_IP: &str = "10.0.0.1";
    pub const SERVER_LAST_SEEN_TIMEOUT: u64 = 60;
    pub static SERVER_USERNAME: &str = "username";
    pub static SERVER_PASSWORD: &str = "password";

    pub static MACHINE_ID: &str = "testmachine";
    pub static MACHINE_NAME: &str = "Test Machine";
    pub static MACHINE_IP: &str = "10.0.0.2";
    pub const MACHINE_LAST_SEEN_TIMEOUT: u64 = 300;

    #[fixture]
    pub fn server_id() -> DeviceId {
        SERVER_ID.parse().unwrap()
    }

    #[fixture]
    pub fn server_ip() -> IpAddr {
        SERVER_IP.parse().unwrap()
    }

    #[fixture]
    pub fn server_mac() -> MacAddr {
        MacAddr::V6(SERVER_MAC.parse().unwrap())
    }

    #[fixture]
    pub fn server() -> Server {
        Server::new(
            &server_id(),
            SERVER_NAME,
            server_ip(),
            SERVER_LAST_SEEN_TIMEOUT,
            server_mac(),
            SERVER_USERNAME,
            Authentication::Password(SERVER_PASSWORD.to_string()),
        )
    }

    #[fixture]
    pub fn machine_id() -> DeviceId {
        MACHINE_ID.parse().unwrap()
    }

    #[fixture]
    pub fn machine_ip() -> IpAddr {
        MACHINE_IP.parse().unwrap()
    }

    #[fixture]
    pub fn machine() -> Machine {
        Machine::new(
            &machine_id(),
            MACHINE_NAME,
            machine_ip(),
            MACHINE_LAST_SEEN_TIMEOUT,
        )
    }
}
