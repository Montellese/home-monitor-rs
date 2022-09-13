use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod api;
mod dependencies;
mod device;
mod files;
mod network;
mod web;

pub use api::Api;
pub use dependencies::{Dependencies, DependencyError};
pub use device::{Device, DeviceId, Machine, Server};
pub use files::Files;
pub use network::{Network, Ping};
pub use web::Web;

pub const LOCATION: &str = "/etc/home-monitor/home-monitor.json";

pub type DeviceMap = HashMap<DeviceId, Device>;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub api: api::Api,
    pub network: network::Network,
    pub devices: DeviceMap,
    pub dependencies: Dependencies,
}

#[allow(dead_code)]
pub fn parse_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Configuration> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Configuration`.
    let mut config: Configuration = serde_json::from_reader(reader)?;

    check_dependencies(&config.devices, &config.dependencies)?;
    fill_ids(&mut config.devices);

    // Return the `Configuration`.
    Ok(config)
}

#[allow(dead_code)]
pub fn parse_from_str(s: &str) -> serde_json::Result<Configuration> {
    // Read the JSON contents of the string as an instance of `Configuration`.
    let mut config: Configuration = serde_json::from_str(s)?;

    check_dependencies(&config.devices, &config.dependencies).unwrap();
    fill_ids(&mut config.devices);

    Ok(config)
}

pub fn fill_ids(devices: &mut DeviceMap) {
    for device in devices.iter_mut() {
        let device_id = device.0.clone();
        match device.1 {
            Device::Server(server) => server.machine.id = device_id,
            Device::Machine(machine) => machine.id = device_id,
        };
    }
}

pub fn get_servers(devices: &DeviceMap) -> HashMap<DeviceId, Server> {
    devices
        .iter()
        .filter_map(|(device_id, device)| match device {
            Device::Server(server) => Some((device_id.clone(), server.clone())),
            _ => None,
        })
        .collect()
}

pub fn get_machines(devices: &DeviceMap) -> HashMap<DeviceId, Machine> {
    devices
        .iter()
        .filter_map(|(device_id, device)| match device {
            Device::Machine(machine) => Some((device_id.clone(), machine.clone())),
            _ => None,
        })
        .collect()
}

fn check_dependencies(
    devices: &DeviceMap,
    dependencies: &Dependencies,
) -> Result<(), DependencyError> {
    if dependencies.0.is_empty() {
        return Err(DependencyError::new(
            "no dependencies configured".to_string(),
        ));
    }

    let servers = get_servers(devices);
    if servers.is_empty() {
        return Err(DependencyError::new("no servers configured".to_string()));
    }

    let machines = get_machines(devices);
    if machines.is_empty() {
        return Err(DependencyError::new("no machines configured".to_string()));
    }

    for (server_id, dependencies) in dependencies.0.iter() {
        // make sure the key of the dependency is a server
        if !servers.contains_key(server_id) {
            return Err(DependencyError::new(format!(
                "{server_id} is not a configured server"
            )));
        }

        // each server needs at least one dependency
        if dependencies.is_empty() {
            return Err(DependencyError::new(format!(
                "{server_id} has no dependencies configured"
            )));
        }

        // make sure the server is not a dependency of itself
        if dependencies.contains(server_id) {
            return Err(DependencyError::new(format!(
                "{server_id} cannot depend on itself"
            )));
        }

        // make sure all values of the dependency exist
        for device_id in dependencies.iter() {
            if !devices.contains_key(device_id) {
                return Err(DependencyError::new(format!(
                    "{server_id} is not a configured device"
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use serde_json::json;

    use super::*;
    use crate::utils::MacAddr;

    static SERVER_ID: &str = "testserver";
    static SERVER_NAME: &str = "Test Server";
    static SERVER_MAC: &str = "aa:bb:cc:dd:ee:ff";
    static SERVER_IP: &str = "10.0.0.1";
    const SERVER_LAST_SEEN_TIMEOUT: u64 = 60;
    static SERVER_USERNAME: &str = "username";
    static SERVER_PASSWORD: &str = "password";

    static MACHINE_ID: &str = "testmachine";
    static MACHINE_NAME: &str = "Test Machine";
    static MACHINE_IP: &str = "10.0.0.2";
    const MACHINE_LAST_SEEN_TIMEOUT: u64 = 300;

    #[fixture]
    fn server_id() -> DeviceId {
        SERVER_ID.parse().unwrap()
    }

    #[fixture]
    fn server() -> Server {
        Server {
            machine: Machine {
                id: server_id(),
                name: SERVER_NAME.to_string(),
                ip: SERVER_IP.parse().unwrap(),
                last_seen_timeout: SERVER_LAST_SEEN_TIMEOUT,
            },
            mac: MacAddr::V6(SERVER_MAC.parse().unwrap()),
            username: SERVER_USERNAME.to_string(),
            password: SERVER_PASSWORD.to_string(),
        }
    }

    #[fixture]
    fn machine_id() -> DeviceId {
        MACHINE_ID.parse().unwrap()
    }

    #[fixture]
    fn machine() -> Machine {
        Machine {
            id: machine_id(),
            name: MACHINE_NAME.to_string(),
            ip: MACHINE_IP.parse().unwrap(),
            last_seen_timeout: MACHINE_LAST_SEEN_TIMEOUT,
        }
    }

    #[rstest]
    fn test_parse_from_file() {
        let mut config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        config_path.push("home-monitor-rs.json.example");

        let config = parse_from_file(config_path);
        assert!(config.is_ok());
    }

    #[rstest]
    fn test_parse_from_str() {
        let config_json = json!({
            "network": {
                "interface": "eth0",
                "ping": {
                    "interval": 6,
                    "timeout": 2
                }
            },
            "api": {
                "files": {
                    "root": "/etc/home-monitor-rs/"
                },
                "web": {
                    "ip": "127.0.0.1",
                    "port": 8000
                }
            },
            "devices": {
                "server1": {
                    "name": "Server 1",
                    "mac": "aa:bb:cc:dd:ee:ff",
                    "ip": "192.168.1.1",
                    "timeout": 60,
                    "username": "foo",
                    "password": "bar"
                },
                "server2": {
                    "name": "Server 2",
                    "mac": "ff:ee:dd:bb:cc:aa",
                    "ip": "192.168.1.129",
                    "timeout": 60,
                    "username": "admin",
                    "password": "1234"
                },
                "mymachine": {
                    "name": "My Machine",
                    "ip": "192.168.1.2",
                    "timeout": 300
                },
                "mywifesmachine": {
                    "id": "mywifesmachine",
                    "name": "My Wife's Machine",
                    "ip": "192.168.1.130",
                    "timeout": 300
                }
            },
            "dependencies": {
                "server1": [
                    "mymachine"
                ],
                "server2": [
                    "server1",
                    "mywifesmachine"
                ]
            }
        });

        let config = parse_from_str(&config_json.to_string());
        assert!(config.is_ok());
    }

    #[rstest]
    fn test_get_servers_is_empty_if_no_servers_configured(machine: Machine) {
        let mut devices = DeviceMap::new();
        devices.insert(machine.id.clone(), Device::Machine(machine));

        assert!(get_servers(&devices).is_empty());
    }

    #[rstest]
    fn test_get_servers_returns_configured_servers(server: Server, machine: Machine) {
        let server_id = server.machine.id.clone();

        let mut devices = DeviceMap::new();
        devices.insert(server_id.clone(), Device::Server(server.clone()));
        devices.insert(machine.id.clone(), Device::Machine(machine));

        let servers = get_servers(&devices);
        assert_eq!(1, servers.len());
        assert!(servers.contains_key(&server_id));
        assert_eq!(server, *servers.get(&server_id).unwrap());
    }

    #[rstest]
    fn test_get_machines_is_empty_if_no_machines_configured(server: Server) {
        let mut devices = DeviceMap::new();
        devices.insert(server.machine.id.clone(), Device::Server(server));

        assert!(get_machines(&devices).is_empty());
    }

    #[rstest]
    fn test_get_machines_returns_configured_machines(server: Server, machine: Machine) {
        let machine_id = machine.id.clone();

        let mut devices = DeviceMap::new();
        devices.insert(server.machine.id.clone(), Device::Server(server.clone()));
        devices.insert(machine_id.clone(), Device::Machine(machine.clone()));

        let machines = get_machines(&devices);
        assert_eq!(1, machines.len());
        assert!(machines.contains_key(&machine_id));
        assert_eq!(machine, *machines.get(&machine_id).unwrap());
    }

    #[rstest]
    fn test_check_dependencies_fails_if_no_dependencies_configured(
        server: Server,
        machine: Machine,
    ) {
        let mut devices = DeviceMap::new();
        devices.insert(server.machine.id.clone(), Device::Server(server.clone()));
        devices.insert(machine.id.clone(), Device::Machine(machine.clone()));

        let dependencies = Dependencies(HashMap::<DeviceId, Vec<DeviceId>>::new());

        assert!(check_dependencies(&devices, &dependencies).is_err());
    }

    #[rstest]
    fn test_check_dependencies_fails_if_no_servers_configured(
        server_id: DeviceId,
        machine: Machine,
    ) {
        let machine_id = machine.id.clone();

        let mut devices = DeviceMap::new();
        devices.insert(machine_id.clone(), Device::Machine(machine));

        let mut dependencies = Dependencies(HashMap::<DeviceId, Vec<DeviceId>>::new());
        dependencies.0.insert(server_id, vec![machine_id.clone()]);

        assert!(check_dependencies(&devices, &dependencies).is_err());
    }

    #[rstest]
    fn test_check_dependencies_fails_if_no_machines_configured(
        server: Server,
        machine_id: DeviceId,
    ) {
        let server_id = server.machine.id.clone();

        let mut devices = DeviceMap::new();
        devices.insert(server_id.clone(), Device::Server(server));

        let mut dependencies = Dependencies(HashMap::<DeviceId, Vec<DeviceId>>::new());
        dependencies.0.insert(server_id.clone(), vec![machine_id]);

        assert!(check_dependencies(&devices, &dependencies).is_err());
    }

    #[rstest]
    fn test_check_dependencies_fails_if_dependency_key_is_not_a_server(
        server: Server,
        machine: Machine,
    ) {
        let server_id = server.machine.id.clone();
        let machine_id = machine.id.clone();

        let mut devices = DeviceMap::new();
        devices.insert(server_id.clone(), Device::Server(server));
        devices.insert(machine_id.clone(), Device::Machine(machine));

        let mut dependencies = Dependencies(HashMap::<DeviceId, Vec<DeviceId>>::new());
        dependencies
            .0
            .insert(machine_id.clone(), vec![server_id.clone()]);

        assert!(check_dependencies(&devices, &dependencies).is_err());
    }

    #[rstest]
    fn test_check_dependencies_fails_if_dependency_key_has_no_dependencies(
        server: Server,
        machine: Machine,
    ) {
        let server_id = server.machine.id.clone();
        let machine_id = machine.id.clone();

        let mut devices = DeviceMap::new();
        devices.insert(server_id.clone(), Device::Server(server));
        devices.insert(machine_id.clone(), Device::Machine(machine));

        let mut dependencies = Dependencies(HashMap::<DeviceId, Vec<DeviceId>>::new());
        dependencies.0.insert(server_id.clone(), vec![]);

        assert!(check_dependencies(&devices, &dependencies).is_err());
    }

    #[rstest]
    fn test_check_dependencies_fails_if_dependency_key_is_also_a_dependency(
        server: Server,
        machine: Machine,
    ) {
        let server_id = server.machine.id.clone();
        let machine_id = machine.id.clone();

        let mut devices = DeviceMap::new();
        devices.insert(server_id.clone(), Device::Server(server));
        devices.insert(machine_id.clone(), Device::Machine(machine));

        let mut dependencies = Dependencies(HashMap::<DeviceId, Vec<DeviceId>>::new());
        dependencies.0.insert(
            server_id.clone(),
            vec![machine_id.clone(), server_id.clone()],
        );

        assert!(check_dependencies(&devices, &dependencies).is_err());
    }

    #[rstest]
    fn test_check_dependencies_fails_if_dependency_doesnt_exist(server: Server, machine: Machine) {
        let server_id = server.machine.id.clone();
        let machine_id = machine.id.clone();

        let mut devices = DeviceMap::new();
        devices.insert(server_id.clone(), Device::Server(server));
        devices.insert(machine_id.clone(), Device::Machine(machine));

        let mut dependencies = Dependencies(HashMap::<DeviceId, Vec<DeviceId>>::new());
        dependencies.0.insert(
            server_id.clone(),
            vec![machine_id.clone(), "badid".parse().unwrap()],
        );

        assert!(check_dependencies(&devices, &dependencies).is_err());
    }

    #[rstest]
    fn test_check_dependencies_succeeds(server: Server, machine: Machine) {
        let server_id = server.machine.id.clone();
        let machine_id = machine.id.clone();

        let mut devices = DeviceMap::new();
        devices.insert(server_id.clone(), Device::Server(server));
        devices.insert(machine_id.clone(), Device::Machine(machine));

        let mut dependencies = Dependencies(HashMap::<DeviceId, Vec<DeviceId>>::new());
        dependencies
            .0
            .insert(server_id.clone(), vec![machine_id.clone()]);

        assert!(check_dependencies(&devices, &dependencies).is_ok());
    }
}
