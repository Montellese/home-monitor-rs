mod config;
mod error;
mod internal_server_error;
mod server;
mod status;

use error::Error;
use internal_server_error::InternalServerError;

pub fn get_routes() -> Vec<rocket::Route> {
    rocket_okapi::openapi_get_routes![
        config::get_config,
        status::get_status,
        server::get_status,
        server::get_always_off,
        server::post_always_off,
        server::delete_always_off,
        server::get_always_on,
        server::post_always_on,
        server::delete_always_on,
        server::put_wakeup,
        server::put_shutdown,
    ]
}
