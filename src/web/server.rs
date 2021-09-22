use log::warn;

pub struct Server {
    name: String,
    version: String,
}

impl Server {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
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

        let server = rocket::custom(&config).launch();
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
