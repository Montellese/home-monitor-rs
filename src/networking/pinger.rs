use std::net::{AddrParseError, IpAddr};
use std::sync::mpsc::RecvError;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Pinger: Send {
    fn add_target(&mut self, ip_addr: IpAddr) -> Result<bool, AddrParseError>;

    fn ping_once(&self);
    fn recv_pong(&mut self) -> Result<(), RecvError>;

    fn is_online(&self, ip_addr: &IpAddr) -> bool;
}
