use std::sync::Arc;

use super::result;
use crate::networking::WakeupServer;

#[rocket::put("/wakeup")]
pub fn put_wakeup(state: &rocket::State<Arc<dyn WakeupServer>>) -> result::Result<()> {
    result::handle(state.wakeup())
}
