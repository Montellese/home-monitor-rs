use std::convert::From;
use std::sync::Arc;

use rocket::serde::json::Json;
use serde::Serialize;

use crate::dom::communication::SharedStateMutex;
use crate::web::api::result;
use crate::web::serialization::Device;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    devices: Vec<Device>,
}

impl Status {
    pub fn new(devices: Vec<Device>) -> Self {
        Self { devices }
    }
}

#[rocket::get("/status")]
pub fn get_status(state: &rocket::State<Arc<SharedStateMutex>>) -> result::Result<Json<Status>> {
    // get the devices from the shared state
    let shared_state = state.lock().unwrap();
    let devices = shared_state.get_devices();

    let status_devices = devices.iter().map(Device::from).collect();

    // create the status response from the devices
    Ok(Json(Status::new(status_devices)))
}
