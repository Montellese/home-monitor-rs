use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Ping {
    pub interval: u64,
    pub timeout: u64,
}

impl Ping {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub interface: String,
    pub ping: Ping,
}

impl Network {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}
