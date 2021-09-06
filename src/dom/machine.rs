use super::super::configuration::machine;

use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Machine {
    pub name: String,
    pub mac: String,
    pub ip: String,

    pub last_seen_timeout: u64,
    pub is_online: bool,
    pub last_seen: Option<Instant>,
}

impl Machine {
    #[allow(dead_code)]
    pub fn new(machine: &machine::Machine) -> Self {
        Machine {
            name: machine.name.clone(),
            mac: machine.mac.clone(),
            ip: machine.ip.clone(),
            last_seen_timeout: machine.last_seen_timeout,
            is_online: false,
            last_seen: None,
        }
    }

    #[allow(dead_code)]
    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
        if online {
            self.last_seen = Some(Instant::now());
        }
    }
}

#[derive(Clone, Debug)]
pub struct Server {
    pub machine: Machine,

    pub username: String,
    pub password: String,
}

impl Server {
    #[allow(dead_code)]
    pub fn new(server: &machine::Server) -> Self {
        Server {
            machine: Machine::new(&server.machine),
            username: server.username.clone(),
            password: server.password.clone(),
        }
    }
}
