use serde::Deserialize;

use std::time::SystemTime;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    pub name: String,
    pub mac: String,
    pub ip: String,
    pub timeout: u16,

    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,

    #[serde(skip)]
    pub is_online: bool,
    #[serde(skip)]
    pub last_seen: Option<SystemTime>,
}

impl Machine {
    #[allow(dead_code)]
    pub fn new() -> Machine {
        Machine::default()
    }

    #[allow(dead_code)]
    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
        if online {
            self.last_seen = Some(SystemTime::now());
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    #[serde(flatten)]
    pub machine: Machine,

    pub username: String,
    pub password: String,
}

impl Server {
    #[allow(dead_code)]
    pub fn new() -> Server {
        Server::default()
    }
}