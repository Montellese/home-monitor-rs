use std::collections::HashMap;

pub mod communication;
pub mod device;

pub use device::{Device, DeviceId, Machine, Server};

pub type Dependencies = HashMap<DeviceId, Vec<DeviceId>>;

#[cfg(test)]
pub mod test {
    use rstest::*;

    use super::device::test::*;
    use super::*;

    #[fixture]
    pub fn dependencies() -> Dependencies {
        [(server_id(), vec![machine_id()])]
            .iter()
            .cloned()
            .collect()
    }
}
