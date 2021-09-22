use crate::configuration::Configuration;

use rocket::serde::json::Json;

#[rocket::get("/config")]
pub fn get_config(state: &rocket::State<Configuration>) -> Json<Configuration> {
    Json(state.inner().clone())
}
