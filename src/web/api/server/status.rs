use std::sync::Arc;

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use super::get_device;
use crate::dom::communication::SharedStateMutex;
use crate::dom::Dependencies;
use crate::web::api::result;
use crate::web::serialization::Device;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    server: Device,
    devices: Vec<Device>,
}

impl Status {
    pub fn new(server: Device, devices: Vec<Device>) -> Self {
        Self { server, devices }
    }
}

#[rocket::get("/server/<server>/status")]
pub fn get_status(
    server: String,
    shared_state: &rocket::State<Arc<SharedStateMutex>>,
    dependencies: &rocket::State<Dependencies>,
) -> result::Result<Json<Status>> {
    // get the devices from the shared state
    let shared_state = shared_state.lock().unwrap();
    let devices = shared_state.get_devices();

    let server_id = server.parse().unwrap();
    // try to find the server
    let server = result::handle_not_found(get_device(devices, &server_id))?;
    // and map it to a serializable device
    let status_server = Device::from(server);

    // get the device IDs of the dependencies
    let dependency_device_ids = dependencies.get(&server_id).unwrap();
    // and map them to the actual device (with status)
    let status_devices = dependency_device_ids
        .iter()
        .map(|device_id| Device::from(get_device(devices, device_id).unwrap()))
        .collect();

    // create the status response from the devices
    Ok(Json(Status::new(status_server, status_devices)))
}

#[cfg(test)]
mod test {
    use std::net::IpAddr;
    use std::sync::Arc;

    use rocket::http::{ContentType, Status};
    use rocket::log::LogLevel;
    use rstest::*;

    use crate::configuration::Configuration;
    use crate::control::test::*;
    use crate::dom::communication::SharedStateMutex;
    use crate::dom::device::test::*;
    use crate::dom::test::*;
    use crate::dom::{Dependencies, DeviceId, Machine, Server};
    use crate::web::api::server::test::*;
    use crate::web::serialization::Device;
    use crate::web::server::test::*;

    #[rstest]
    fn test_web_api_can_get_server_status(
        config: Configuration,
        shared_state: Arc<SharedStateMutex>,
        mocked_server_control: MockServerControl,
        dependencies: Dependencies,
        ip: IpAddr,
        port: u16,
        log_level: LogLevel,
        server_id: DeviceId,
        server: Server,
        machine: Machine,
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

        let response = client
            .get(get_server_api_endpoint("/status", &server_id))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let expected_status = super::Status::new(Device::from(server), vec![Device::from(machine)]);
        assert_eq!(response.into_json::<super::Status>(), Some(expected_status));
    }

    #[rstest]
    fn test_web_api_cannot_get_invalid_server_status(
        config: Configuration,
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

        let response = client
            .get(get_server_api_endpoint(
                "/status",
                &"invalidserverid".parse().unwrap(),
            ))
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }
}
