use super::result;

use crate::utils::AlwaysOff;

use rocket::serde::json::Json;
use serde::Serialize;

use std::sync::Arc;

#[derive(Serialize)]
pub struct AlwaysOffResponse {
    always_off: bool,
}

#[rocket::get("/always_off")]
pub fn get_always_off(state: &rocket::State<Arc<dyn AlwaysOff>>) -> Json<AlwaysOffResponse> {
    Json(AlwaysOffResponse {
        always_off: state.is_always_off(),
    })
}

#[rocket::post("/always_off")]
pub fn post_always_off(
    state: &rocket::State<Arc<dyn AlwaysOff>>,
) -> result::Result<Json<AlwaysOffResponse>> {
    match result::handle(state.set_always_off()) {
        Ok(_) => Ok(Json(AlwaysOffResponse { always_off: true })),
        Err(e) => Err(e),
    }
}

#[rocket::delete("/always_off")]
pub fn delete_always_off(
    state: &rocket::State<Arc<dyn AlwaysOff>>,
) -> result::Result<Json<AlwaysOffResponse>> {
    match result::handle(state.reset_always_off()) {
        Ok(_) => Ok(Json(AlwaysOffResponse { always_off: false })),
        Err(e) => Err(e),
    }
}
