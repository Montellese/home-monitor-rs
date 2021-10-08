use std::convert::From;
use std::sync::Arc;

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::dom::communication::SharedStateMutex;
use crate::web::api::result;
use crate::web::serialization::Device;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    devices: Vec<Device>,
}

impl Status {
    pub fn new(devices: Vec<Device>) -> Self {
        Self { devices }
    }
}

#[rocket::get("/status")]
pub fn get_status(state: &rocket::State<Arc<SharedStateMutex>>) -> result::Result<Json<Status>> {
    // get the devices from the shared state
    let shared_state = state.lock().unwrap();
    let devices = shared_state.get_devices();

    let status_devices = devices.iter().map(Device::from).collect();

    // create the status response from the devices
    Ok(Json(Status::new(status_devices)))
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
    use crate::dom::test::*;
    use crate::dom::Dependencies;
    use crate::web::serialization;
    use crate::web::server::test::*;

    #[rstest]
    fn test_web_api_can_get_status(
        config: Configuration,
        shared_state: Arc<SharedStateMutex>,
        mocked_server_control: MockServerControl,
        dependencies: Dependencies,
        ip: IpAddr,
        port: u16,
        log_level: LogLevel,
        serialization_devices: Vec<serialization::Device>,
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

        let response = client.get(get_api_endpoint("/status")).dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let expected_status = super::Status::new(serialization_devices);
        assert_eq!(response.into_json::<super::Status>(), Some(expected_status));
    }
}
