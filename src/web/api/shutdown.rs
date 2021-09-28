use super::result;

use crate::networking::ShutdownServer;

use std::sync::Arc;

#[rocket::put("/shutdown")]
pub fn put_shutdown(state: &rocket::State<Arc<dyn ShutdownServer>>) -> result::Result<()> {
    result::handle(state.shutdown())
}
