use anyhow::anyhow;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use super::get_server_control;
use crate::control::ServerControl;
use crate::web::api::result;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AlwaysOffResponse {
    always_off: bool,
}

#[rocket::get("/server/<server>/always_off")]
pub fn get_always_off(
    server: String,
    state: &rocket::State<Vec<ServerControl>>,
) -> result::Result<Json<AlwaysOffResponse>> {
    match result::handle_not_found(get_server_control(state.inner(), server)) {
        Ok(control) => Ok(Json(AlwaysOffResponse {
            always_off: control.always_off.is_always_off(),
        })),
        Err(e) => Err(e),
    }
}

#[rocket::post("/server/<server>/always_off")]
pub fn post_always_off(
    server: String,
    state: &rocket::State<Vec<ServerControl>>,
) -> result::Result<Json<AlwaysOffResponse>> {
    let control = result::handle_not_found(get_server_control(state.inner(), server))?;

    match result::handle_internal_server_error(control.always_off.set_always_off()) {
        Ok(_) => Ok(Json(AlwaysOffResponse { always_off: true })),
        Err(e) => Err(e),
    }
}

#[rocket::delete("/server/<server>/always_off")]
pub fn delete_always_off(
    server: String,
    state: &rocket::State<Vec<ServerControl>>,
) -> result::Result<Json<AlwaysOffResponse>> {
    let control = result::handle_not_found(get_server_control(state.inner(), server))?;

    match result::handle_internal_server_error(control.always_off.reset_always_off()) {
        Ok(_) => Ok(Json(AlwaysOffResponse { always_off: false })),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod test {
    use std::io::{Error, ErrorKind};
    use std::net::IpAddr;
    use std::sync::Arc;

    use rocket::http::{ContentType, Status};
    use rocket::log::LogLevel;
    use rstest::*;

    use super::*;
    use crate::configuration::Configuration;
    use crate::control::test::*;
    use crate::dom::communication::SharedStateMutex;
    use crate::dom::device::test::*;
    use crate::dom::test::*;
    use crate::dom::{Dependencies, DeviceId};
    use crate::web::api::server::test::*;
    use crate::web::server::test::*;

    #[rstest]
    fn test_web_api_can_get_always_off(
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
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| true);

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
            .get(get_server_api_endpoint("/always_off", &server_id))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        assert_eq!(
            response.into_json::<AlwaysOffResponse>(),
            Some(AlwaysOffResponse { always_off: true })
        );
    }

    #[rstest]
    fn test_web_api_cannot_get_always_off_for_invalid_server(
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
                "/always_off",
                &"invalidserverid".parse().unwrap(),
            ))
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    #[rstest]
    fn test_web_api_can_set_always_off(
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
            .always_off
            .expect_set_always_off()
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
            .post(get_server_api_endpoint("/always_off", &server_id))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        assert_eq!(
            response.into_json::<AlwaysOffResponse>(),
            Some(AlwaysOffResponse { always_off: true })
        );
    }

    #[rstest]
    fn test_web_api_returns_internal_server_error_if_set_always_off_fails(
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
            .always_off
            .expect_set_always_off()
            .once()
            .return_once(|| Err(anyhow!(Error::new(ErrorKind::PermissionDenied, ""))));

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
            .post(get_server_api_endpoint("/always_off", &server_id))
            .dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
    }

    #[rstest]
    fn test_web_api_cannot_set_always_off_for_invalid_server(
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
            .post(get_server_api_endpoint(
                "/always_off",
                &"invalidserverid".parse().unwrap(),
            ))
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    #[rstest]
    fn test_web_api_can_delete_always_off(
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
            .always_off
            .expect_reset_always_off()
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
            .delete(get_server_api_endpoint("/always_off", &server_id))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        assert_eq!(
            response.into_json::<AlwaysOffResponse>(),
            Some(AlwaysOffResponse { always_off: false })
        );
    }

    #[rstest]
    fn test_web_api_returns_internal_server_error_if_delete_always_off_fails(
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
            .always_off
            .expect_reset_always_off()
            .once()
            .return_once(|| Err(anyhow!(Error::new(ErrorKind::PermissionDenied, ""))));

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
            .delete(get_server_api_endpoint("/always_off", &server_id))
            .dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
    }

    #[rstest]
    fn test_web_api_cannot_delete_always_off_for_invalid_server(
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
            .delete(get_server_api_endpoint(
                "/always_off",
                &"invalidserverid".parse().unwrap(),
            ))
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }
}
