use super::result;

use crate::dom;
use crate::dom::communication;
use crate::dom::communication::SharedStateMutex;

use chrono::{DateTime, Utc};
use log::error;
use rocket::serde::json::Json;
use serde::Serialize;

use std::convert::From;
use std::option::Option;
use std::sync::Arc;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub name: String,
    pub mac: String,
    pub ip: String,

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
            mac: machine.mac.clone(),
            ip: machine.ip.clone(),
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
        Device::from(&server.machine)
    }
}

impl From<communication::Device> for Device {
    fn from(device: communication::Device) -> Self {
        Device::from(&device)
    }
}

impl From<&communication::Device> for Device {
    fn from(device: &communication::Device) -> Self {
        match device {
            communication::Device::Server(server) => Device::from(server),
            communication::Device::Machine(machine) => Device::from(machine),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    server: Device,
    machines: Vec<Device>,
}

impl Status {
    pub fn new(server: Device, machines: Vec<Device>) -> Self {
        Self { server, machines }
    }
}

#[rocket::get("/status")]
pub fn get_status(state: &rocket::State<Arc<SharedStateMutex>>) -> result::Result<Json<Status>> {
    // get the devices from the shared state
    let devices = state.lock().unwrap().get_devices();

    let mut status_server: Option<Device> = None;
    let mut status_machines: Vec<Device> = Vec::with_capacity(devices.len() - 1);

    for device in devices {
        match device {
            communication::Device::Server(server) => match status_server {
                Some(ref device) => {
                    error!(
                        "received a status for more than one server: {:?} != {:?}",
                        device,
                        Device::from(server)
                    );
                }
                None => {
                    status_server = Some(Device::from(server));
                }
            },
            communication::Device::Machine(machine) => status_machines.push(Device::from(machine)),
        }
    }

    // create the status response from the machines
    match status_server {
        Some(server) => Ok(Json(Status::new(server, status_machines))),
        None => result::handle(Err("Invalid shared state without a server".to_string())),
    }
}
