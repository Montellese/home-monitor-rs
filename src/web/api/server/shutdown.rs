use super::get_server_control;
use crate::control::ServerControl;
use crate::web::api::result;

#[rocket::put("/server/<server>/shutdown")]
pub fn put_shutdown(
    server: String,
    state: &rocket::State<Vec<ServerControl>>,
) -> result::Result<()> {
    let control = result::handle_not_found(get_server_control(state.inner(), server))?;

    result::handle_internal_server_error(control.shutdown.shutdown())
}

#[cfg(test)]
mod test {
    use std::net::IpAddr;
    use std::sync::Arc;

    use rocket::http::Status;
    use rocket::log::LogLevel;
    use rstest::*;

    use crate::configuration::Configuration;
    use crate::control::test::*;
    use crate::dom::communication::SharedStateMutex;
    use crate::dom::device::test::*;
    use crate::dom::test::*;
    use crate::dom::{Dependencies, DeviceId};
    use crate::networking::ShutdownError;
    use crate::web::api::server::test::*;
    use crate::web::server::test::*;

    #[rstest]
    fn test_web_api_can_shutdown_server(
        config: Configuration,
        shared_state: Arc<SharedStateMutex>,
        mut mocked_server_control: MockServerControl,
        dependencies: Dependencies,
        ip: IpAddr,
        port: u16,
        log_level: LogLevel,
        server_id: DeviceId,
    ) {
        // EXPECTATIONS
        mocked_server_control
            .shutdown
            .expect_shutdown()
            .once()
            .return_once(|| Ok(()));

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
            .put(get_server_api_endpoint("/shutdown", &server_id))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
    }

    #[rstest]
    fn test_web_api_returns_internal_server_error_if_shutdown_server_fails(
        config: Configuration,
        shared_state: Arc<SharedStateMutex>,
        mut mocked_server_control: MockServerControl,
        dependencies: Dependencies,
        ip: IpAddr,
        port: u16,
        log_level: LogLevel,
        server_id: DeviceId,
    ) {
        // EXPECTATIONS
        mocked_server_control
            .shutdown
            .expect_shutdown()
            .once()
            .return_once(|| Err(ShutdownError::new("".to_string())));

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
            .put(get_server_api_endpoint("/shutdown", &server_id))
            .dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
    }

    #[rstest]
    fn test_web_api_cannot_shutdown_invalid_server(
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
            .put(get_server_api_endpoint(
                "/shutdown",
                &"invalidserverid".parse().unwrap(),
            ))
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }
}
