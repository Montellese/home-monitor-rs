use rocket::serde::json::Json;
use serde::Serialize;

use super::get_server_control;
use crate::control::ServerControl;
use crate::web::api::result;

#[derive(Serialize)]
pub struct AlwaysOnResponse {
    always_on: bool,
}

#[rocket::get("/server/<server>/always_on")]
pub fn get_always_on(
    server: String,
    state: &rocket::State<Vec<ServerControl>>,
) -> result::Result<Json<AlwaysOnResponse>> {
    match result::handle_not_found(get_server_control(state.inner(), server)) {
        Ok(control) => Ok(Json(AlwaysOnResponse {
            always_on: control.always_on.is_always_on(),
        })),
        Err(e) => Err(e),
    }
}

#[rocket::post("/server/<server>/always_on")]
pub fn post_always_on(
    server: String,
    state: &rocket::State<Vec<ServerControl>>,
) -> result::Result<Json<AlwaysOnResponse>> {
    let control = result::handle_not_found(get_server_control(state.inner(), server))?;

    match result::handle_internal_server_error(control.always_on.set_always_on()) {
        Ok(_) => Ok(Json(AlwaysOnResponse { always_on: true })),
        Err(e) => Err(e),
    }
}

#[rocket::delete("/server/<server>/always_on")]
pub fn delete_always_on(
    server: String,
    state: &rocket::State<Vec<ServerControl>>,
) -> result::Result<Json<AlwaysOnResponse>> {
    let control = result::handle_not_found(get_server_control(state.inner(), server))?;

    match result::handle_internal_server_error(control.always_on.reset_always_on()) {
        Ok(_) => Ok(Json(AlwaysOnResponse { always_on: false })),
        Err(e) => Err(e),
    }
}
