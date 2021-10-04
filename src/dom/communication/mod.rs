mod mpsc_sender;
mod noop_sender;
mod sender;
mod shared_state;

pub use mpsc_sender::MpscSender;
pub use noop_sender::NoopSender;
#[cfg(test)]
pub use sender::MockSender;
pub use sender::Sender;
pub use shared_state::{SharedState, SharedStateMutex};

pub type MpscReceiver = tokio::sync::mpsc::UnboundedReceiver<super::Device>;

pub fn mpsc_channel() -> (MpscSender, MpscReceiver) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<super::Device>();

    (MpscSender::new(tx), rx)
}

pub fn create_mpsc_sender(mpsc_sender: MpscSender) -> Box<dyn Sender> {
    Box::new(mpsc_sender)
}

pub fn create_noop_sender() -> Box<dyn Sender> {
    Box::new(NoopSender::new())
}
