mod device;
mod mpsc_sender;
mod sender;

pub use device::Device;
pub use mpsc_sender::MpscSender;
#[cfg(test)]
pub use sender::MockSender;
pub use sender::Sender;

pub type MpscReceiver = tokio::sync::mpsc::UnboundedReceiver<Device>;

pub fn mpsc_channel() -> (MpscSender, MpscReceiver) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Device>();

    (MpscSender::new(tx), rx)
}
