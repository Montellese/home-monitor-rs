use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ping {
    interval: u32,
    timeout: u32,
}

impl Ping {
    #[allow(dead_code)]
    pub fn new() -> Ping {
        Ping::default()
    }

    #[allow(dead_code)]
    pub fn interval(&self) -> &u32 {
        &self.interval
    }

    #[allow(dead_code)]
    pub fn timeout(&self) -> &u32 {
        &self.timeout
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    interface: String,
    ping: Ping,
}

impl Network {
    #[allow(dead_code)]
    pub fn new() -> Network {
        Network::default()
    }

    #[allow(dead_code)]
    pub fn interface(&self) -> &String {
        &self.interface
    }

    #[allow(dead_code)]
    pub fn ping(&self) -> &Ping {
        &self.ping
    }
}
