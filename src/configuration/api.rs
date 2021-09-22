use super::Files;
use super::Web;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
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
