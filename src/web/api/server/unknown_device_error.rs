use std::fmt;

use crate::dom::DeviceId;

#[derive(Debug)]
pub struct UnknownDeviceError(DeviceId);

impl UnknownDeviceError {
    pub fn new(device_id: DeviceId) -> Self {
        Self(device_id)
    }
}

impl std::error::Error for UnknownDeviceError {}

impl fmt::Display for UnknownDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[UnknownDeviceError] {}", self.0)
    }
}
