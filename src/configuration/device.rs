use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use super::super::utils::MacAddr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    pub name: String,
    pub ip: IpAddr,

    #[serde(rename = "timeout")]
    pub last_seen_timeout: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    #[serde(flatten)]
    pub machine: Machine,

    pub mac: MacAddr,
    pub username: String,
    pub password: String,
}
