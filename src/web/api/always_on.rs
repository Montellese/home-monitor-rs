use crate::utils::AlwaysOn;

use rocket::http::Status;
use rocket::serde::json::Json;
use serde::Serialize;

use std::sync::Arc;

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
) -> Result<Json<AlwaysOnResponse>, (Status, String)> {
    match state.set_always_on() {
        Ok(_) => Ok(Json(AlwaysOnResponse { always_on: true })),
        Err(e) => Err((Status::InternalServerError, e.to_string())),
    }
}

#[rocket::delete("/always_on")]
pub fn delete_always_on(
    state: &rocket::State<Arc<dyn AlwaysOn>>,
) -> Result<Json<AlwaysOnResponse>, (Status, String)> {
    match state.reset_always_on() {
        Ok(_) => Ok(Json(AlwaysOnResponse { always_on: false })),
        Err(e) => Err((Status::InternalServerError, e.to_string())),
    }
}
