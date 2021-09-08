pub trait WakeupServer {
    fn wakeup(&self) -> std::io::Result<()>;
}
