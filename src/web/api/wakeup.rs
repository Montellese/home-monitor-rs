use super::result;

use crate::networking::WakeupServer;

use std::sync::Arc;

#[rocket::put("/wakeup")]
pub fn put_wakeup(state: &rocket::State<Arc<dyn WakeupServer>>) -> result::Result<()> {
    result::handle(state.wakeup())
}
