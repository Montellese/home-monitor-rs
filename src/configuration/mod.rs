use serde::Deserialize;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub mod files;
pub mod logging;
pub mod machine;
pub mod network;

pub const LOCATION: &'static str = "/etc/home-monitor/home-monitor.json";

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    logging: logging::Logging,
    files: files::Files,
    network: network::Network,
    server: machine::Server,
    machines: Vec<machine::Machine>,
}

impl Configuration {
    #[allow(dead_code)]
    pub fn logging(&self) -> &logging::Logging {
        &self.logging
    }

    #[allow(dead_code)]
    pub fn files(&self) -> &files::Files {
        &self.files
    }

    #[allow(dead_code)]
    pub fn network(&self) -> &network::Network {
        &self.network
    }

    #[allow(dead_code)]
    pub fn server(&self) -> &machine::Server {
        &self.server
    }

    #[allow(dead_code)]
    pub fn machines(&self) -> &Vec<machine::Machine> {
        &self.machines
    }
}

#[allow(dead_code)]
pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<Configuration, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Configuration`.
    let config = serde_json::from_reader(reader)?;

    // Return the `Configuration`.
    Ok(config)
}

#[allow(dead_code)]
pub fn parse_from_str(s: &str) -> serde_json::Result<Configuration> {
    // Read the JSON contents of the string as an instance of `Configuration`.
    serde_json::from_str(s)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_parse_from_str() {
        let config_json = r#"{
            "logging": {
                "level": "DEBUG"
            },
            "files": {
                "alwaysOn": "/etc/home-monitor/alwayson"
            },
            "network": {
                "interface": "eth0",
                "ping": {
                    "interval": 6,
                    "timeout": 2
                }
            },
            "server": {
                "name": "My Server",
                "mac": "aa:bb:cc:dd:ee:ff",
                "ip": "192.168.1.1",
                "timeout": 60,
                "username": "foo",
                "password": "bar"
            },
            "machines": [
                {
                    "name": "My Machine",
                    "mac": "ff:ee:dd:cc:bb:aa",
                    "ip": "192.168.1.2",
                    "timeout": 300
                }
            ]
        }"#;

        let config = parse_from_str(config_json);
        match config {
            Err(e) => assert!(false, format!("{:#?}", e)),
            _ => (),
        };
    }
}
