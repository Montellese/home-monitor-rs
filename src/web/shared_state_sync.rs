use crate::dom::communication::{MpscReceiver, SharedStateMutex};

use log::debug;

use std::sync::Arc;

pub struct SharedStateSync {
    shared_state: Arc<SharedStateMutex>,
    receiver: MpscReceiver,
}

impl SharedStateSync {
    pub fn new(shared_state: Arc<SharedStateMutex>, receiver: MpscReceiver) -> Self {
        Self {
            shared_state,
            receiver,
        }
    }

    pub async fn sync(&mut self) {
        loop {
            match self.receiver.recv().await {
                Some(updated_device) => {
                    debug!("updating {} in shared state", updated_device);
                    self.shared_state
                        .lock()
                        .unwrap()
                        .update_device(updated_device);
                }
                None => {
                    debug!("stopping shared state sync because all senders were dropped");
                    break;
                }
            }
        }
    }
}
