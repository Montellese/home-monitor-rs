use std::net::IpAddr;
use std::sync::mpsc::RecvError;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Pinger: Send {
    fn add_target(&mut self, ip_addr: IpAddr) -> bool;

    fn ping_once(&self);
    fn recv_pong(&mut self) -> Result<(), RecvError>;

    fn is_online(&self, ip_addr: &IpAddr) -> bool;
}
