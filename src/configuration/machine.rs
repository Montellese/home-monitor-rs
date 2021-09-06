use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    pub name: String,
    pub mac: String,
    pub ip: String,

    #[serde(rename = "timeout")]
    pub last_seen_timeout: u64,
}

impl Machine {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Machine::default()
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
    pub fn new() -> Self {
        Server::default()
    }
}
