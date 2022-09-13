use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::utils::MacAddr;

#[derive(
    Clone, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, JsonSchema,
)]
pub struct DeviceId(pub String);

impl FromStr for DeviceId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    #[serde(skip)]
    pub id: DeviceId,
    pub name: String,
    pub ip: IpAddr,

    #[serde(rename = "timeout")]
    pub last_seen_timeout: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    #[serde(flatten)]
    pub machine: Machine,

    pub mac: MacAddr,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum Device {
    Server(Server),
    Machine(Machine),
}
