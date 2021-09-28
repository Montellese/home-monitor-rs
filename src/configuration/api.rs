use super::Files;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Api {
    pub files: Files,
}

impl Api {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}
