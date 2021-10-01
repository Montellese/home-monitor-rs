use std::sync::Arc;

use log::warn;

use super::api;
use crate::configuration::Configuration;
use crate::control::ServerControl;
use crate::dom::communication::SharedStateMutex;
use crate::dom::Dependencies;

pub struct Server {
    name: String,
    version: String,
    config: Configuration,

    shared_state: Arc<SharedStateMutex>,
    server_controls: Vec<ServerControl>,
    dependencies: Dependencies,
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
    ) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            config,
            shared_state,
            server_controls,
            dependencies,
        }
    }

    pub async fn launch(
        &self,
        ip: std::net::IpAddr,
        port: u16,
        log_level: rocket::config::LogLevel,
    ) -> std::result::Result<(), rocket::Error> {
        // create a custom configuration for Rocket
        let mut config = rocket::Config {
            address: ip,
            port,
            log_level,
            cli_colors: false,
            ..Default::default()
        };

        // configure the "Server" identity
        match rocket::config::Ident::try_new(format!("{}/{}", self.name, self.version)) {
            Ok(ident) => config.ident = ident,
            Err(e) => warn!("failed to create custom identitiy for the web API: {}", e),
        };

        let server = rocket::custom(&config)
            .mount(
                "/api/v1/",
                rocket::routes![
                    api::get_config,
                    api::get_status,
                    api::server::get_status,
                    api::server::get_always_off,
                    api::server::post_always_off,
                    api::server::delete_always_off,
                    api::server::get_always_on,
                    api::server::post_always_on,
                    api::server::delete_always_on,
                    api::server::put_wakeup,
                    api::server::put_shutdown,
                ],
            )
            .manage(self.config.clone())
            .manage(self.shared_state.clone())
            .manage(self.server_controls.clone())
            .manage(self.dependencies.clone())
            .launch();
        server.await
    }

    pub fn get_num_workers() -> usize {
        rocket::Config::from(rocket::Config::figment()).workers
    }

    pub fn get_thread_name(name: &str) -> String {
        // NOTE: graceful shutdown of tokio runtime depends on the "rocket-worker" prefix.
        format!("rocket-worker-{}", name)
    }
}
