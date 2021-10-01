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
    type Err = Box<dyn std::error::Error>;

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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Server {
    pub machine: Machine,

    pub mac: MacAddr,
    pub username: String,
    pub password: String,
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
        password: &str,
    ) -> Self {
        Self {
            machine: Machine::new(id, name, ip, last_seen_timeout),
            mac,
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}

impl From<&configuration::Server> for Server {
    fn from(server: &configuration::Server) -> Self {
        Self {
            machine: Machine::from(&server.machine),
            mac: server.mac,
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

#[derive(Clone, Debug)]
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
