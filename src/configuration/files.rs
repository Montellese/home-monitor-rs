use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Files {
    pub root: PathBuf,
}

impl Files {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}
