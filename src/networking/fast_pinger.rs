use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::mpsc::{Receiver, RecvError};

use fastping_rs::PingResult;
use fastping_rs::PingResult::{Idle, Receive};
use log::warn;

use super::Pinger;

pub struct FastPinger {
    pinger: fastping_rs::Pinger,
    pinger_results: Receiver<PingResult>,

    targets: HashMap<IpAddr, bool>,
}

impl FastPinger {
    pub fn new(max_rtt: Option<u64>) -> Self {
        let (pinger, pinger_results) = match fastping_rs::Pinger::new(max_rtt, None) {
            Ok((pinger, results)) => (pinger, results),
            Err(e) => panic!("Failed to create fastping_rs::Pinger: {}", e),
        };

        Self {
            pinger,
            pinger_results,
            targets: HashMap::<IpAddr, bool>::new(),
        }
    }

    fn set_online(&mut self, ip_addr: &IpAddr, is_online: bool) {
        let target_is_online = match self.targets.get_mut(ip_addr) {
            Some(target) => target,
            None => {
                warn!("received unexpected pong for {}", ip_addr);
                return;
            }
        };

        *target_is_online = is_online;
    }

    fn ip_to_string(ip_addr: &IpAddr) -> String {
        format!("{ip_addr}")
    }
}

impl Pinger for FastPinger {
    fn add_target(&mut self, ip_addr: IpAddr) -> bool {
        // only add the target IP address if it doesn't already exist
        if self.targets.get(&ip_addr).is_none() {
            self.pinger
                .add_ipaddr(Self::ip_to_string(&ip_addr).as_str());
            self.targets.insert(ip_addr, false);

            true
        } else {
            false
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
            #[allow(clippy::question_mark)]
            if let Err(e) = result {
                return Err(e);
            }
        }

        Ok(())
    }

    fn is_online(&self, ip_addr: &IpAddr) -> bool {
        match self.targets.get(ip_addr) {
            Some(is_online) => *is_online,
            None => false,
        }
    }
}
