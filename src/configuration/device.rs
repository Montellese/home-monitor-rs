use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    pub name: String,
    pub ip: String,

    #[serde(rename = "timeout")]
    pub last_seen_timeout: u64,
}

impl Machine {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    #[serde(flatten)]
    pub machine: Machine,

    pub mac: String,
    pub username: String,
    pub password: String,
}

impl Server {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}