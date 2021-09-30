use std::sync::Arc;

use super::result;
use crate::networking::ShutdownServer;

#[rocket::put("/shutdown")]
pub fn put_shutdown(state: &rocket::State<Arc<dyn ShutdownServer>>) -> result::Result<()> {
    result::handle(state.shutdown())
}
