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
