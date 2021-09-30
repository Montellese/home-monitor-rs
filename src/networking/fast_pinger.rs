use std::collections::HashMap;
use std::net::{AddrParseError, IpAddr};
use std::sync::mpsc::{Receiver, RecvError};

use fastping_rs::PingResult;
use fastping_rs::PingResult::{Idle, Receive};
use log::{error, warn};

use super::Pinger;

pub struct FastPinger {
    pinger: fastping_rs::Pinger,
    pinger_results: Receiver<PingResult>,

    targets: HashMap<String, bool>,
}

impl FastPinger {
    pub fn new(max_rtt: Option<u64>) -> Self {
        let (pinger, pinger_results) = match fastping_rs::Pinger::new(max_rtt, None) {
            Ok((pinger, results)) => (pinger, results),
            Err(e) => panic!("Failed to create fastping_rs::Pinger: {}", e),
        };

        let targets: HashMap<String, bool> = HashMap::new();

        Self {
            pinger,
            pinger_results,
            targets,
        }
    }

    fn set_online(&mut self, ip_addr: &IpAddr, is_online: bool) {
        let addr = Self::ip_to_string(ip_addr);

        let target_is_online = match self.targets.get_mut(&addr) {
            Some(target) => target,
            None => {
                warn!("received unexpected pong for {}", addr);
                return;
            }
        };

        *target_is_online = is_online;
    }

    fn normalize_ip(ip_addr: &str) -> Result<String, AddrParseError> {
        let addr: IpAddr = ip_addr.parse()?;

        Ok(Self::ip_to_string(&addr))
    }

    fn ip_to_string(ip_addr: &IpAddr) -> String {
        format!("{}", ip_addr)
    }
}

impl Pinger for FastPinger {
    fn add_target(&mut self, ip_addr: &str) -> Result<bool, AddrParseError> {
        // normalize the IP address
        let addr = Self::normalize_ip(ip_addr)?;

        // only add the target IP address if it doesn't already exist
        if self.targets.get(&addr).is_none() {
            self.pinger.add_ipaddr(&addr);
            self.targets.insert(addr, false);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn ping_once(&self) {
        self.pinger.ping_once()
    }

    fn recv_pong(&mut self) -> Result<(), RecvError> {
        let len = self.targets.len();
        for _ in 0..len {
            let result = match self.pinger_results.recv() {
                Ok(result) => match result {
                    Idle { addr } => {
                        self.set_online(&addr, false);
                        Ok(false)
                    }
                    Receive { addr, .. } => {
                        self.set_online(&addr, true);
                        Ok(true)
                    }
                },
                Err(e) => Err(e),
            };

            // early return on an error
            if let Err(e) = result {
                return Err(e);
            }
        }

        Ok(())
    }

    fn is_online(&self, ip_addr: &str) -> bool {
        // normalize the IP address
        let addr = match Self::normalize_ip(ip_addr) {
            Ok(normalized_addr) => normalized_addr,
            Err(e) => {
                error!("failed to parse IP address {}: {}", ip_addr, e);
                return false;
            }
        };

        match self.targets.get(&addr) {
            Some(is_online) => *is_online,
            None => false,
        }
    }
}
