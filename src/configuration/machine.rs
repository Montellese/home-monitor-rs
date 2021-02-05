use serde::Deserialize;

use std::time::SystemTime;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    name: String,
    mac: String,
    ip: String,
    timeout: u16,

    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,

    #[serde(skip)]
    is_online: bool,
    #[serde(skip)]
    last_seen: Option<SystemTime>,
}

impl Machine {
    #[allow(dead_code)]
    pub fn new() -> Machine {
        Machine::default()
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &String {
        &self.name
    }

    #[allow(dead_code)]
    pub fn mac(&self) -> &String {
        &self.mac
    }

    #[allow(dead_code)]
    pub fn ip(&self) -> &String {
        &self.ip
    }

    #[allow(dead_code)]
    pub fn username(&self) -> &String {
        &self.username
    }

    #[allow(dead_code)]
    pub fn password(&self) -> &String {
        &self.password
    }

    #[allow(dead_code)]
    pub fn timeout(&self) -> &u16 {
        &self.timeout
    }

    #[allow(dead_code)]
    pub fn is_online(&self) -> &bool {
        &self.is_online
    }

    #[allow(dead_code)]
    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
        if online {
            self.last_seen = Some(SystemTime::now());
        }
    }

    #[allow(dead_code)]
    pub fn last_seen(&self) -> &Option<SystemTime> {
        &self.last_seen
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    #[serde(flatten)]
    machine: Machine,

    username: String,
    password: String,
}

impl Server {
    #[allow(dead_code)]
    pub fn new() -> Server {
        Server::default()
    }

    #[allow(dead_code)]
    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    #[allow(dead_code)]
    pub fn username(&self) -> &String {
        &self.username
    }

    #[allow(dead_code)]
    pub fn password(&self) -> &String {
        &self.password
    }
}