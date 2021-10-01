use rocket::serde::json::Json;
use serde::Serialize;

use super::get_server_control;
use crate::control::ServerControl;
use crate::web::api::result;

#[derive(Serialize)]
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
