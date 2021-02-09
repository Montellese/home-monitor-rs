use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ping {
    pub interval: u64,
    pub timeout: u64,
}

impl Ping {
    #[allow(dead_code)]
    pub fn new() -> Ping {
        Ping::default()
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub interface: String,
    pub ping: Ping,
}

impl Network {
    #[allow(dead_code)]
    pub fn new() -> Network {
        Network::default()
    }
}
