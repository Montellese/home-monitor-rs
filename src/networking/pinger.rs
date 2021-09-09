use std::net::AddrParseError;
use std::sync::mpsc::RecvError;

pub trait Pinger {
    fn add_target(&mut self, ip_addr: &str) -> Result<bool, AddrParseError>;

    fn ping_once(&self);
    fn recv_pong(&mut self) -> Result<(), RecvError>;

    fn is_online(&self, ip_addr: &str) -> bool;
}