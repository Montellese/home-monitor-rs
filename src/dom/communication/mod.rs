mod mpsc_sender;
mod sender;
mod shared_state;

pub use mpsc_sender::MpscSender;
#[cfg(test)]
pub use sender::MockSender;
pub use sender::Sender;
pub use shared_state::{SharedState, SharedStateMutex};

pub type MpscReceiver = tokio::sync::mpsc::UnboundedReceiver<super::Device>;

pub fn mpsc_channel() -> (MpscSender, MpscReceiver) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<super::Device>();

    (MpscSender::new(tx), rx)
}
