mod always_off;
mod always_on;
mod shutdown;
mod status;
mod unknown_device_error;
mod wakeup;

pub use always_off::{delete_always_off, get_always_off, post_always_off};
pub use always_on::{delete_always_on, get_always_on, post_always_on};
pub use shutdown::put_shutdown;
pub use status::get_status;
pub use unknown_device_error::UnknownDeviceError;
pub use wakeup::put_wakeup;

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
        None => Err(UnknownDeviceError::new(server_id)),
    }
}

fn get_device<'a>(
    devices: &'a [crate::dom::Device],
    device_id: &crate::dom::DeviceId,
) -> Result<&'a crate::dom::Device, UnknownDeviceError> {
    match devices.iter().find(|device| device.id() == device_id) {
        Some(device) => Ok(device),
        None => Err(UnknownDeviceError::new(device_id.clone())),
    }
}
