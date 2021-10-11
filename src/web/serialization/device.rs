use std::convert::From;
use std::net::IpAddr;
use std::option::Option;

use macaddr::MacAddr8;
use rocket_okapi::JsonSchema;
use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Schema, SchemaObject};
use serde::{Deserialize, Serialize};

use crate::dom;
use crate::utils::MacAddr;

impl JsonSchema for MacAddr {
    fn schema_name() -> String {
        "macaddr".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("macaddr".to_owned()),
            ..Default::default()
        }
        .into()
    }

    fn is_referenceable() -> bool {
        false
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub name: String,
    pub ip: IpAddr,
    #[serde(default = "Device::default_mac")]
    #[serde(skip_serializing_if = "MacAddr::is_nil")]
    pub mac: MacAddr,

    pub last_seen_timeout: u64,
    pub is_online: bool,
    pub last_seen: Option<String>,
}

impl Device {
    pub fn default_mac() -> MacAddr {
        MacAddr::V8(MacAddr8::nil())
    }
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
            mac: Self::default_mac(),
            last_seen_timeout: machine.last_seen_timeout,
            is_online: machine.is_online,
            last_seen: machine.last_seen_date.map(|date| date.to_string()),
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
