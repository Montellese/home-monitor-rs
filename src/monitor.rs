use super::configuration::Configuration;
use super::dom::always_on::AlwaysOn;
use super::dom::machine::{Machine, Server};
use super::networking::controllable_server::ControllableServer;

use fastping_rs::PingResult;
use fastping_rs::PingResult::{Idle, Receive};
use fastping_rs::Pinger;

use log::{debug, error, info, warn};

use std::collections::HashMap;
use std::net::IpAddr;
use std::ops::Sub;
use std::time::{Duration, Instant};

const CHANGE_TIMEOUT: Duration = Duration::from_secs(120);

pub struct Monitor<'a> {
    controllable_server: Box<dyn ControllableServer>,
    server: Server<'a>,
    server_ip: IpAddr,
    machines: HashMap<IpAddr, Machine<'a>>,

    always_on_state: bool,
    always_on: Box<dyn AlwaysOn>,
    last_ping: Instant,
    last_change: Instant,
    ping_interval: Duration,

    pinger: Pinger,
    pinger_results: std::sync::mpsc::Receiver<PingResult>,
}

impl<'a> Monitor<'a> {
    pub fn new(
        config: &'a Configuration,
        controllable_server: Box<dyn ControllableServer>,
        always_on: Box<dyn AlwaysOn>,
    ) -> Self {
        let ping_interval = Duration::from_secs(config.network.ping.interval);

        // create a pinger and its results receiver
        let (pinger, pinger_results) = match Pinger::new(None, None) {
            Ok((pinger, results)) => (pinger, results),
            Err(e) => panic!("Failed to create pinger: {}", e),
        };

        let mut machines: HashMap<IpAddr, Machine> = HashMap::new();

        // add the IP address of the server and all machines to the pinger
        pinger.add_ipaddr(&config.server.machine.ip);
        for machine in config.machines.iter() {
            let machine_ip = &machine.ip;

            match machine_ip.parse() {
                Ok(ip_addr) => {
                    machines.insert(ip_addr, Machine::new(machine));
                    pinger.add_ipaddr(machine_ip);
                }
                Err(e) => {
                    error!("failed to parse IP address of {}: {}", machine.name, e);
                }
            }
        }

        let server_ip: IpAddr = match config.server.machine.ip.parse() {
            Ok(ip_addr) => ip_addr,
            Err(e) => {
                panic!(
                    "Failed to parse server IP address ({}): {}",
                    config.server.machine.ip, e
                );
            }
        };

        Monitor {
            controllable_server,
            server: Server::new(&config.server),
            server_ip,
            machines,
            always_on_state: false,
            always_on,
            last_ping: Instant::now().sub(ping_interval),
            last_change: Instant::now().sub(CHANGE_TIMEOUT),
            ping_interval,
            pinger,
            pinger_results,
        }
    }

    pub fn run_once(&mut self) {
        // check the always on file
        {
            let always_on_file_exists = self.always_on.is_always_on();
            if always_on_file_exists != self.always_on_state {
                if always_on_file_exists {
                    info!("ALWAYS ON has been enabled");
                } else {
                    info!("ALWAYS ON has been disabled");
                }

                self.always_on_state = always_on_file_exists;
            }
        }

        // check if the server is online
        if self.last_ping.elapsed() > self.ping_interval {
            self.last_ping = Instant::now();

            // determine the number of machines (+ server)
            let num_machines = self.machines.len() + 1;

            // run the pinger once
            debug!("pinging {} machines...", num_machines);
            self.pinger.ping_once();

            // receive all results
            for _ in 0..num_machines {
                match self.pinger_results.recv() {
                    Ok(result) => match result {
                        Idle { addr } => {
                            let machine: Option<&mut Machine>;
                            if addr == self.server_ip {
                                machine = Some(&mut self.server.machine);
                            } else {
                                machine = self.machines.get_mut(&addr);
                            }

                            match machine {
                                Some(machine) => {
                                    debug!(
                                        "no ping response from {} ({})",
                                        machine.name, machine.ip
                                    );

                                    // update the online state of the machine
                                    Monitor::check_and_update_machine_online(machine, false);
                                }
                                None => {
                                    warn!("received unexpected ping idle for {}", addr);
                                }
                            };
                        }
                        Receive { addr, .. } => {
                            let machine: Option<&mut Machine>;
                            if addr == self.server_ip {
                                machine = Some(&mut self.server.machine);
                            } else {
                                machine = self.machines.get_mut(&addr);
                            }

                            match machine {
                                Some(machine) => {
                                    debug!(
                                        "received ping response from {} ({})",
                                        machine.name, machine.ip
                                    );

                                    // update the online state of the machine
                                    Monitor::check_and_update_machine_online(machine, true);
                                }
                                None => {
                                    warn!("received unexpected ping response from {}", addr);
                                }
                            };
                        }
                    },
                    Err(e) => panic!("Pinger failed: {}", e),
                };
            }
        }

        // check if any machine is online
        let any_machine_is_online = self.machines.values().any(|machine| machine.is_online);

        // process the collected information
        if self.always_on_state || self.last_change.elapsed() > CHANGE_TIMEOUT {
            let server = &self.server;

            // if the server is not online and
            //   the always on file exists or
            //   any machine is online
            // then wake the server up
            if !self.server.machine.is_online && (self.always_on_state || any_machine_is_online) {
                info!("waking up {}...", server.machine.name);
                match self.controllable_server.wakeup() {
                    Err(_) => error!("failed to wake up {}", server.machine.name),
                    Ok(_) => {
                        self.last_change = Instant::now();
                    }
                }
            } else if !self.always_on_state && !any_machine_is_online && server.machine.is_online {
                info!("shutting down {}...", server.machine.name);
                match self.controllable_server.shutdown() {
                    Err(e) => error!("failed to shut down {}: {}", server.machine.name, e),
                    Ok(_) => {
                        self.last_change = Instant::now();
                    }
                }
            }
        }
    }

    fn check_and_update_machine_online(machine: &mut Machine, is_online: bool) {
        let machine_was_online = machine.is_online;

        // update the servers online state
        //   either if it is currently online
        //   or if it has become offline
        if is_online {
            machine.set_online(true)
        } else if machine_was_online
            && machine.last_seen.unwrap().elapsed() > Duration::from_secs(machine.last_seen_timeout)
        {
            machine.set_online(false)
        }

        if is_online != machine_was_online {
            if is_online {
                info!("{} ({}) is now online", machine.name, machine.ip);
            } else {
                info!("{} ({}) is now offline", machine.name, machine.ip);
            }
        }
    }
}
