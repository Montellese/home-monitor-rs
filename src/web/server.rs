use std::sync::Arc;

use log::warn;

use super::api::*;
use crate::configuration::Configuration;
use crate::dom::communication::SharedStateMutex;
use crate::networking::{ShutdownServer, WakeupServer};
use crate::utils::{AlwaysOff, AlwaysOn};

pub struct Server {
    name: String,
    version: String,
    config: Configuration,

    shared_state: Arc<SharedStateMutex>,

    always_off: Arc<dyn AlwaysOff>,
    always_on: Arc<dyn AlwaysOn>,

    wakeup_server: Arc<dyn WakeupServer>,
    shutdown_server: Arc<dyn ShutdownServer>,
}

impl Server {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        version: &str,
        config: Configuration,
        shared_state: Arc<SharedStateMutex>,
        always_off: Arc<dyn AlwaysOff>,
        always_on: Arc<dyn AlwaysOn>,
        wakeup_server: Arc<dyn WakeupServer>,
        shutdown_server: Arc<dyn ShutdownServer>,
    ) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            config,
            shared_state,
            always_off,
            always_on,
            wakeup_server,
            shutdown_server,
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
                    get_config,
                    get_status,
                    get_always_off,
                    post_always_off,
                    delete_always_off,
                    get_always_on,
                    post_always_on,
                    delete_always_on,
                    put_wakeup,
                    put_shutdown,
                ],
            )
            .manage(self.config.clone())
            .manage(self.shared_state.clone())
            .manage(self.always_off.clone())
            .manage(self.always_on.clone())
            .manage(self.wakeup_server.clone())
            .manage(self.shutdown_server.clone())
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
