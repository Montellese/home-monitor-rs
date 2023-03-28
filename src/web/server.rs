use std::net::IpAddr;
use std::sync::Arc;

use log::warn;
use rocket_okapi::rapidoc::{
    make_rapidoc, GeneralConfig, HideShowConfig, LayoutConfig, NavConfig, RapiDocConfig,
    RenderStyle, Theme, UiConfig,
};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

use super::api;
use crate::configuration::Configuration;
use crate::control::ServerControl;
use crate::dom::communication::SharedStateMutex;
use crate::dom::Dependencies;
use crate::env::PKG_NAME;

static OPENAPI_SPEC: &str = "/api/v1/openapi.json";

fn swagger_ui() -> SwaggerUIConfig {
    SwaggerUIConfig {
        url: OPENAPI_SPEC.to_string(),
        urls: vec![],
        ..Default::default()
    }
}

fn rapidoc() -> RapiDocConfig {
    RapiDocConfig {
        general: GeneralConfig {
            spec_urls: vec![UrlObject::new(PKG_NAME, OPENAPI_SPEC)],
            sort_tags: true,
            ..Default::default()
        },
        ui: UiConfig {
            theme: Theme::Dark,
            ..Default::default()
        },
        nav: NavConfig {
            use_path_in_nav_bar: true,
            ..Default::default()
        },
        layout: LayoutConfig {
            render_style: RenderStyle::Read,
            ..Default::default()
        },
        hide_show: HideShowConfig {
            show_info: false,
            allow_authentication: false,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub struct Server {
    server: rocket::Rocket<rocket::Build>,
}

impl Server {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        version: &str,
        config: Configuration,
        shared_state: Arc<SharedStateMutex>,
        server_controls: Vec<ServerControl>,
        dependencies: Dependencies,
        ip: IpAddr,
        port: u16,
        log_level: rocket::config::LogLevel,
    ) -> Self {
        // create a custom configuration for Rocket
        let mut rocket_config = rocket::Config {
            address: ip,
            port,
            log_level,
            cli_colors: false,
            ..Default::default()
        };

        // configure the "Server" identity
        match rocket::config::Ident::try_new(format!("{name}/{version}")) {
            Ok(ident) => rocket_config.ident = ident,
            Err(e) => warn!("failed to create custom identitiy for the web API: {}", e),
        };

        let server = rocket::custom(&rocket_config)
            .mount("/api/v1/", api::get_routes())
            .mount("/docs/swagger/", make_swagger_ui(&swagger_ui()))
            .mount("/docs/rapidoc/", make_rapidoc(&rapidoc()))
            .manage(config)
            .manage(shared_state)
            .manage(server_controls)
            .manage(dependencies);

        Self { server }
    }

    pub async fn launch(
        self,
    ) -> std::result::Result<rocket::Rocket<rocket::Ignite>, rocket::Error> {
        self.server.launch().await
    }

    pub fn get_num_workers() -> usize {
        rocket::Config::from(rocket::Config::figment()).workers
    }

    pub fn get_thread_name(name: &str) -> String {
        // NOTE: graceful shutdown of tokio runtime depends on the "rocket-worker" prefix.
        format!("rocket-worker-{name}")
    }

    #[cfg(test)]
    fn rocket(self) -> rocket::Rocket<rocket::Build> {
        self.server
    }
}

#[cfg(test)]
pub mod test {
    use std::sync::Mutex;

    use rocket::local::blocking::Client;
    use rocket::log::LogLevel;
    use rstest::*;
    use serde_json::json;

    use super::*;
    use crate::control::test::*;
    use crate::dom::device::test::*;
    use crate::env::*;
    use crate::web::serialization;
    use crate::{configuration, dom};

    #[fixture]
    pub fn devices(server: dom::Server, machine: dom::Machine) -> Vec<dom::Device> {
        vec![dom::Device::Server(server), dom::Device::Machine(machine)]
    }

    #[fixture]
    pub fn serialization_devices(
        server: dom::Server,
        machine: dom::Machine,
    ) -> Vec<serialization::Device> {
        vec![
            serialization::Device::from(server),
            serialization::Device::from(machine),
        ]
    }

    #[fixture]
    pub fn config(server: dom::Server, machine: dom::Machine) -> Configuration {
        let password = match &server.ssh.authentication {
            dom::device::SshAuthentication::Password(pw) => pw,
            dom::device::SshAuthentication::PrivateKey(pk) => &pk.passphrase,
        };

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
                    "root": "/etc/home-monitor-rs/",
                },
                "web": {
                    "ip": "127.0.0.1",
                    "port": 8000
                }
            },
            "devices": {
                server.machine.id.to_string(): {
                    "name": server.machine.name,
                    "mac": server.mac,
                    "ip": server.machine.ip,
                    "timeout": server.machine.last_seen_timeout,
                    "ssh": {
                        "port": Into::<u16>::into(server.ssh.port),
                        "username": server.ssh.username,
                        "password": password
                    }
                },
                machine.id.to_string(): {
                    "name": machine.name,
                    "ip": machine.ip,
                    "timeout": machine.last_seen_timeout
                },
            },
            "dependencies": {
                server.machine.id.to_string(): [
                    machine.id.to_string()
                ]
            }
        });

        let config = configuration::parse_from_str(&config_json.to_string());
        assert!(config.is_ok());

        config.unwrap()
    }

    #[fixture]
    pub fn shared_state(devices: Vec<dom::Device>) -> Arc<SharedStateMutex> {
        Arc::new(Mutex::new(dom::communication::SharedState::new(devices)))
    }

    #[fixture]
    pub fn ip(config: Configuration) -> IpAddr {
        config.api.web.ip
    }

    #[fixture]
    pub fn port(config: Configuration) -> u16 {
        config.api.web.port
    }

    #[fixture]
    pub fn log_level() -> LogLevel {
        rocket::log::LogLevel::Debug
    }

    pub fn get_client(
        config: &Configuration,
        shared_state: Arc<SharedStateMutex>,
        mocked_server_control: MockServerControl,
        dependencies: Dependencies,
        ip: IpAddr,
        port: u16,
        log_level: LogLevel,
    ) -> Client {
        let server = Server::new(
            PKG_NAME,
            PKG_VERSION,
            config.clone(),
            shared_state,
            vec![ServerControl::from(mocked_server_control)],
            dependencies,
            ip,
            port,
            log_level,
        );

        Client::tracked(server.rocket()).unwrap()
    }

    pub fn get_api_endpoint(endpoint: &str) -> String {
        format!("/api/v1{endpoint}")
    }
}
