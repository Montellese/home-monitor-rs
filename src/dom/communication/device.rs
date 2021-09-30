use std::fmt;

use super::super::{Machine, Server};

#[derive(Clone, Debug)]
pub enum Device {
    Server(Server),
    Machine(Machine),
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Device::Server(ref server) => write!(f, "{}", server),
            Device::Machine(ref machine) => write!(f, "{}", machine),
        }
    }
}
