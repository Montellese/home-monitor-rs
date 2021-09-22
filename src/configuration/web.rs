use serde::{Deserialize, Serialize};

use std::net::IpAddr;

#[derive(Clone, Debug, Deserialize, Serialize)]
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
