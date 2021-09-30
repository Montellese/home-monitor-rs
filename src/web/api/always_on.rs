use std::sync::Arc;

use rocket::serde::json::Json;
use serde::Serialize;

use super::result;
use crate::utils::AlwaysOn;

#[derive(Serialize)]
pub struct AlwaysOnResponse {
    always_on: bool,
}

#[rocket::get("/always_on")]
pub fn get_always_on(state: &rocket::State<Arc<dyn AlwaysOn>>) -> Json<AlwaysOnResponse> {
    Json(AlwaysOnResponse {
        always_on: state.is_always_on(),
    })
}

#[rocket::post("/always_on")]
pub fn post_always_on(
    state: &rocket::State<Arc<dyn AlwaysOn>>,
) -> result::Result<Json<AlwaysOnResponse>> {
    match result::handle(state.set_always_on()) {
        Ok(_) => Ok(Json(AlwaysOnResponse { always_on: true })),
        Err(e) => Err(e),
    }
}

#[rocket::delete("/always_on")]
pub fn delete_always_on(
    state: &rocket::State<Arc<dyn AlwaysOn>>,
) -> result::Result<Json<AlwaysOnResponse>> {
    match result::handle(state.reset_always_on()) {
        Ok(_) => Ok(Json(AlwaysOnResponse { always_on: false })),
        Err(e) => Err(e),
    }
}
