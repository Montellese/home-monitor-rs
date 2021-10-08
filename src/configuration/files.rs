use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
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
