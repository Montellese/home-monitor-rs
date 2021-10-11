use std::net::IpAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Web {
    #[serde(default = "Web::default_ip")]
    pub ip: IpAddr,
    #[serde(default)]
    pub port: u16,
}

impl Web {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default_ip() -> IpAddr {
        "0.0.0.0".parse().unwrap()
    }
}

impl Default for Web {
    fn default() -> Self {
        Self {
            ip: Web::default_ip(),
            port: 0,
        }
    }
}
