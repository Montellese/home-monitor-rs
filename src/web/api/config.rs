use rocket::get;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

use crate::configuration::Configuration;

#[openapi(tag = "General")]
#[get("/config")]
pub fn get_config(state: &rocket::State<Configuration>) -> Json<Configuration> {
    Json(state.inner().clone())
}

#[cfg(test)]
mod test {
    use std::net::IpAddr;
    use std::sync::Arc;

    use rocket::http::{ContentType, Status};
    use rocket::log::LogLevel;
    use rstest::*;

    use crate::configuration;
    use crate::configuration::Configuration;
    use crate::control::test::*;
    use crate::dom::communication::SharedStateMutex;
    use crate::dom::test::*;
    use crate::dom::Dependencies;
    use crate::web::server::test::*;

    #[rstest]
    fn test_web_api_can_get_config(
        mut config: Configuration,
        shared_state: Arc<SharedStateMutex>,
        mocked_server_control: MockServerControl,
        dependencies: Dependencies,
        ip: IpAddr,
        port: u16,
        log_level: LogLevel,
    ) {
        // TESTING
        let client = get_client(
            &config,
            shared_state,
            mocked_server_control,
            dependencies,
            ip,
            port,
            log_level,
        );

        let response = client.get(get_api_endpoint("/config")).dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        // we need to reset the DeviceIds in config because it isn't serialized / deserialized
        for device in config.devices.iter_mut() {
            match device.1 {
                configuration::Device::Server(server) => server.machine.id.0.clear(),
                configuration::Device::Machine(machine) => machine.id.0.clear(),
            };
        }
        assert_eq!(response.into_json::<Configuration>(), Some(config));
    }
}
