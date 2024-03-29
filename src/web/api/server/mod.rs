mod always_off;
mod always_on;
mod shutdown;
mod status;
mod unknown_device_error;
mod wakeup;

pub use always_off::*;
pub use always_on::*;
pub use shutdown::*;
pub use status::*;
pub use unknown_device_error::UnknownDeviceError;
pub use wakeup::*;

use crate::dom::{Device, DeviceId};

fn get_server_control(
    servers: &[crate::control::ServerControl],
    server_id: String,
) -> Result<&crate::control::ServerControl, UnknownDeviceError> {
    let server_id = server_id.parse().unwrap();
    match servers
        .iter()
        .find(|control| control.server.machine.id == server_id)
    {
        Some(control) => Ok(control),
        None => Err(UnknownDeviceError::from(server_id)),
    }
}

fn get_device<'a>(
    devices: &'a [Device],
    device_id: &DeviceId,
) -> Result<&'a Device, UnknownDeviceError> {
    match devices.iter().find(|device| device.id() == device_id) {
        Some(device) => Ok(device),
        None => Err(UnknownDeviceError::from(device_id.clone())),
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub fn get_server_api_endpoint(endpoint: &str, server_id: &DeviceId) -> String {
        format!(
            "/api/v1/server/{server_id}{endpoint}",
            server_id = server_id.to_string()
        )
    }
}
