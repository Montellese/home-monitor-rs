use std::collections::HashMap;

pub mod communication;
mod device;

pub use device::{Device, DeviceId, Machine, Server};

pub type Dependencies = HashMap<DeviceId, Vec<DeviceId>>;
