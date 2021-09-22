use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Files {
    pub always_on: String,
    pub always_off: String,
}

impl Files {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}
