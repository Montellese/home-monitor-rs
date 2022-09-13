use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Files, Web};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Api {
    pub files: Files,
    #[serde(default)]
    pub web: Web,
}

impl Api {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}
