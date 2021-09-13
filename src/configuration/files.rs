use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
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
