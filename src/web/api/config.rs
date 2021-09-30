use rocket::serde::json::Json;

use crate::configuration::Configuration;

#[rocket::get("/config")]
pub fn get_config(state: &rocket::State<Configuration>) -> Json<Configuration> {
    Json(state.inner().clone())
}
