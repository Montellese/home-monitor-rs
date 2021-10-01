use std::sync::Arc;

use rocket::serde::json::Json;
use serde::Serialize;

use super::get_device;
use crate::dom::communication::SharedStateMutex;
use crate::dom::Dependencies;
use crate::web::api::result;
use crate::web::serialization::Device;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    server: Device,
    devices: Vec<Device>,
}

impl Status {
    pub fn new(server: Device, devices: Vec<Device>) -> Self {
        Self { server, devices }
    }
}

#[rocket::get("/server/<server>/status")]
pub fn get_status(
    server: String,
    shared_state: &rocket::State<Arc<SharedStateMutex>>,
    dependencies: &rocket::State<Dependencies>,
) -> result::Result<Json<Status>> {
    // get the devices from the shared state
    let shared_state = shared_state.lock().unwrap();
    let devices = shared_state.get_devices();

    let server_id = server.parse().unwrap();
    // try to find the server
    let server = result::handle_not_found(get_device(devices, &server_id))?;
    // and map it to a serializable device
    let status_server = Device::from(server);

    // get the device IDs of the dependencies
    let dependency_device_ids = dependencies.get(&server_id).unwrap();
    // and map them to the actual device (with status)
    let status_devices = dependency_device_ids
        .iter()
        .map(|device_id| Device::from(get_device(devices, device_id).unwrap()))
        .collect();

    // create the status response from the devices
    Ok(Json(Status::new(status_server, status_devices)))
}
