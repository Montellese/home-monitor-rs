use super::configuration::{Configuration, machine};
use super::networking::{ping, shutdown, wakeup};

use log::{debug, error, info, warn};

use std::ops::Sub;
use std::path::Path;
use std::time::{Duration, Instant};

const CHANGE_TIMEOUT: Duration = Duration::from_secs(120);

pub struct Monitor {
    config: Configuration,

    always_on: bool,
    last_ping: Instant,
    last_change: Instant,
    ping_interval: Duration,
}

impl Monitor {
    pub fn new(config: Configuration) -> Monitor {
        let ping_interval = Duration::from_secs(config.network.ping.interval);
        Monitor {
            always_on: false,
            last_ping: Instant::now().sub(ping_interval),
            last_change: Instant::now().sub(CHANGE_TIMEOUT),
            ping_interval: ping_interval,
            config: config,
        }
    }

    pub fn run_once(&mut self) {
        // check the always on file
        {
            let always_on_file_exists = Path::new(&self.config.files.always_on).exists();
            if always_on_file_exists != self.always_on {
                if always_on_file_exists {
                    info!("ALWAYS ON has been enabled");
                } else {
                    info!("ALWAYS ON has been disabled");
                }

                self.always_on = always_on_file_exists;
            }
        }

        // check if the server is online
        if self.last_ping.elapsed() > self.ping_interval {
            self.last_ping = Instant::now();

            // update the online state of the server
            Monitor::check_and_update_machine_online(&mut self.config.server.machine);

            // check if any machine is online
            for mut machine in self.config.machines.iter_mut() {
                Monitor::check_and_update_machine_online(&mut machine);
            }
        }

        // check if any machine is online
        let any_machine_is_online = self.config.machines.iter().any(|machine| machine.is_online);

        // process the collected information
        if self.always_on || self.last_change.elapsed() > CHANGE_TIMEOUT {
            let server = &self.config.server;

            // if the server is not online and
            //   the always on file exists or
            //   any machine is online
            // then wake the server up
            if !self.config.server.machine.is_online && (self.always_on || any_machine_is_online) {
                info!("waking up {}...", server.machine.name);
                match wakeup(&server.machine) {
                    Err(_)=> error!("failed to wake up {}", server.machine.name),
                    Ok(_) => {
                        self.last_change = Instant::now();
                        info!("{} successfully woken up", server.machine.name);
                    },
                }
            } else if !self.always_on && !any_machine_is_online && server.machine.is_online {
                info!("shutting down {}...", server.machine.name);
                match shutdown::shutdown(&server) {
                    Err(e)=> error!("failed to shut down {}: {}", server.machine.name, e),
                    Ok(_) => {
                        self.last_change = Instant::now();
                        info!("{} successfully shut down", server.machine.name);
                    },
                }
            }
        }
    }

    fn check_and_update_machine_online(machine: &mut machine::Machine) {
        // check if the machine is online
        debug!("checking if {} ({}) is online...", machine.name, machine.ip);
        let machine_is_online = match ping::ping(&machine.ip) {
            Err(e) => {
                warn!("failed to ping {} ({}): {}", machine.name, machine.ip, e);
                false
            },
            Ok(r) => r,
        };

        let machine_was_online = machine.is_online;

        // update the servers online state
        //   either if it is currently online
        //   or if it has become offline
        if machine_is_online {
            machine.set_online(true)
        } else if machine_was_online && machine.last_seen.unwrap().elapsed() >  Duration::from_secs(machine.last_seen_timeout) {
            machine.set_online(false)
        }

        if machine_is_online != machine_was_online {
            if machine_is_online {
                info!("{} ({}) is now online", machine.name, machine.ip);
            } else {
                info!("{} ({}) is now offline", machine.name, machine.ip);
            }
        }
    }
}
