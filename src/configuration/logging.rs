use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Traice
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logging {
    level: Level,
}

impl Logging {
    #[allow(dead_code)]
    pub fn new() -> Logging {
        Logging::default()
    }

    #[allow(dead_code)]
    pub fn level(&self) -> &Level {
        &self.level
    }
}

impl Default for Logging {
    fn default() -> Logging {
        Logging {
            level: Level::Info,
        }
    }
}
