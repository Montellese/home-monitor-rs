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
        if machine.ip == updated_machine.ip {
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
