use std::collections::HashMap;
use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::DeviceId;

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct Dependencies(pub HashMap<DeviceId, Vec<DeviceId>>);

#[derive(Debug, Clone)]
pub struct DependencyError(String);

impl DependencyError {
    pub fn new(error_msg: String) -> Self {
        Self(error_msg)
    }
}

impl std::error::Error for DependencyError {}

impl fmt::Display for DependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[DependencyError] {}", self.0)
    }
}
