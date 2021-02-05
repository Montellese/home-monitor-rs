use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Files {
    always_on: String,
}

impl Files {
    #[allow(dead_code)]
    pub fn new() -> Files {
        Files::default()
    }

    #[allow(dead_code)]
    pub fn always_on(&self) -> &String {
        &self.always_on
    }
}
