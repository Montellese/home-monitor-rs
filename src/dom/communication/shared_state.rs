use super::super::{Device, Machine, Server};

pub struct SharedState {
    devices: Vec<Device>,
}

impl SharedState {
    pub fn new(devices: Vec<Device>) -> Self {
        Self { devices }
    }

    pub fn get_devices(&self) -> &Vec<Device> {
        &self.devices
    }

    pub fn update_device(&mut self, device: Device) {
        // try to find a matching machine by IP and update the mutable fields
        for dev in self.devices.iter_mut() {
            if match device {
                Device::Server(ref server) => Self::update_device_from_server(dev, server),
                Device::Machine(ref machine) => Self::update_device_from_machine(dev, machine),
            } {
                // early return
                return;
            }
        }

        // otherwise add the machine to the shared state
        self.devices.push(device);
    }

    fn update_device_from_server(device: &mut Device, updated_server: &Server) -> bool {
        // only update a server device with a server
        match device {
            Device::Server(ref mut server) => {
                Self::raw_update_machine_from_machine(&mut server.machine, &updated_server.machine)
            }
            _ => false,
        }
    }

    fn update_device_from_machine(device: &mut Device, updated_machine: &Machine) -> bool {
        // only update a machine device with a machine
        match device {
            Device::Machine(ref mut machine) => {
                Self::raw_update_machine_from_machine(machine, updated_machine)
            }
            _ => false,
        }
    }

    fn raw_update_machine_from_machine(machine: &mut Machine, updated_machine: &Machine) -> bool {
        if machine.id == updated_machine.id {
            machine.is_online = updated_machine.is_online;
            machine.last_seen = updated_machine.last_seen;
            machine.last_seen_date = updated_machine.last_seen_date;

            true
        } else {
            false
        }
    }
}

pub type SharedStateMutex = std::sync::Mutex<SharedState>;

#[cfg(test)]
mod test {
    use rstest::*;

    use super::*;
    use crate::dom::device::test::*;

    #[fixture]
    fn devices(server: Server, machine: Machine) -> Vec<Device> {
        vec![Device::Server(server), Device::Machine(machine)]
    }

    #[fixture]
    fn shared_state(devices: Vec<Device>) -> SharedState {
        SharedState::new(devices)
    }

    #[rstest]
    fn test_can_get_devices(shared_state: SharedState, devices: Vec<Device>) {
        assert_eq!(*shared_state.get_devices(), devices);
    }

    #[rstest]
    fn test_update_devices_adds_device_if_not_existing(
        mut shared_state: SharedState,
        mut devices: Vec<Device>,
    ) {
        // SETUP
        let mut new_device = devices.last().unwrap().clone();
        match new_device {
            Device::Server(ref mut server) => server.machine.id = "newserver".parse().unwrap(),
            Device::Machine(ref mut machine) => machine.id = "newmachine".parse().unwrap(),
        };

        // TESTING
        assert_eq!(*shared_state.get_devices(), devices);

        shared_state.update_device(new_device.clone());
        devices.push(new_device);

        assert_eq!(*shared_state.get_devices(), devices);
    }

    #[rstest]
    fn test_can_update_existing_device(mut shared_state: SharedState, mut devices: Vec<Device>) {
        // TESTING
        assert_eq!(*shared_state.get_devices(), devices);

        let device = devices.last_mut().unwrap();
        match device {
            Device::Server(ref mut server) => server.machine.is_online = !server.machine.is_online,
            Device::Machine(ref mut machine) => machine.is_online = !machine.is_online,
        };

        shared_state.update_device(device.clone());

        assert_eq!(*shared_state.get_devices(), devices);
    }
}
