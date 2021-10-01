use std::convert::From;
use std::net::IpAddr;
use std::option::Option;

use chrono::{DateTime, Utc};
use macaddr::MacAddr8;
use serde::Serialize;

use crate::dom;
use crate::utils::MacAddr;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub name: String,
    pub ip: IpAddr,
    #[serde(default)]
    #[serde(skip_serializing_if = "MacAddr::is_nil")]
    pub mac: MacAddr,

    pub last_seen_timeout: u64,
    pub is_online: bool,
    pub last_seen: Option<DateTime<Utc>>,
}

impl From<dom::Machine> for Device {
    fn from(machine: dom::Machine) -> Self {
        Device::from(&machine)
    }
}

impl From<&dom::Machine> for Device {
    fn from(machine: &dom::Machine) -> Self {
        Self {
            name: machine.name.clone(),
            ip: machine.ip,
            mac: MacAddr::V8(MacAddr8::nil()),
            last_seen_timeout: machine.last_seen_timeout,
            is_online: machine.is_online,
            last_seen: machine.last_seen_date,
        }
    }
}

impl From<dom::Server> for Device {
    fn from(server: dom::Server) -> Self {
        Device::from(&server)
    }
}

impl From<&dom::Server> for Device {
    fn from(server: &dom::Server) -> Self {
        let mut device = Device::from(&server.machine);
        device.mac = server.mac;

        device
    }
}

impl From<dom::Device> for Device {
    fn from(device: dom::Device) -> Self {
        Device::from(&device)
    }
}

impl From<&dom::Device> for Device {
    fn from(device: &dom::Device) -> Self {
        match device {
            dom::Device::Server(server) => Device::from(server),
            dom::Device::Machine(machine) => Device::from(machine),
        }
    }
}
