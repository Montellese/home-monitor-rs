use super::dom::machine::{Machine, Server};
use super::networking::pinger::Pinger;
use super::networking::shutdown_server::ShutdownServer;
use super::networking::wakeup_server::WakeupServer;
use super::utils::always_on::AlwaysOn;
use super::utils::Instant;

use log::{debug, error, info, warn};

use std::ops::Sub;
use std::time::Duration;

const CHANGE_TIMEOUT: Duration = Duration::from_secs(120);

pub struct Monitor {
    server: Server,
    machines: Vec<Machine>,

    wakeup_server: Box<dyn WakeupServer>,
    shutdown_server: Box<dyn ShutdownServer>,

    always_on_state: bool,
    always_on: Box<dyn AlwaysOn>,
    last_ping: Instant,
    last_change: Instant,
    ping_interval: Duration,

    pinger: Box<dyn Pinger>,
}

impl Monitor {
    pub fn new(
        ping_interval: Duration,
        server: Server,
        machines: Vec<Machine>,
        wakeup_server: Box<dyn WakeupServer>,
        shutdown_server: Box<dyn ShutdownServer>,
        pinger: Box<dyn Pinger>,
        always_on: Box<dyn AlwaysOn>,
    ) -> Self {
        // get a mutable binding to pinger
        let mut mut_pinger = pinger;

        // add the IP address of the server to the pinger
        match mut_pinger.add_target(&server.machine.ip) {
            Ok(false) => {
                panic!("failed to add {} to the pinger", server)
            }
            Err(e) => {
                panic!("failed to parse IP address of {}: {}", server, e);
            }
            _ => (),
        }

        let mut mut_machines = machines;

        // add the IP address of all machines to the pinger
        mut_machines.retain(|machine: &Machine| {
            return match mut_pinger.add_target(&machine.ip) {
                Ok(true) => true,
                Ok(false) => {
                    warn!("failed to add {} to the pinger", machine);
                    false
                }
                Err(e) => {
                    error!("failed to parse IP address of {}: {}", machine, e);
                    false
                }
            };
        });

        if mut_machines.is_empty() {
            panic!("no machines to monitor {}", server);
        }

        let now = Instant::now();
        let last_ping = now.sub(ping_interval);
        let last_change = now.sub(CHANGE_TIMEOUT);

        Monitor {
            server,
            machines: mut_machines,
            wakeup_server,
            shutdown_server,
            always_on_state: false,
            always_on,
            last_ping,
            last_change,
            ping_interval,
            pinger: mut_pinger,
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
            // and receive all responses (pongs)
            if let Err(e) = self.pinger.recv_pong() {
                panic!("Pinger failed to receive responses: {}", e)
            }

            {
                // update the online state of the server
                let server_is_online = self.pinger.is_online(&self.server.machine.ip);
                Self::update_machine_online(&mut self.server.machine, server_is_online);
            }

            // update the online state of all machines
            for mut machine in self.machines.iter_mut() {
                let is_online = self.pinger.is_online(&machine.ip);
                Self::update_machine_online(&mut machine, is_online);
            }
        }

        // check if any machine is online
        let any_machine_is_online = self.machines.iter().any(|machine| machine.is_online);

        // process the collected information
        if self.always_on_state || self.last_change.elapsed() > CHANGE_TIMEOUT {
            // if the server is not online and
            //   the always on file exists or
            //   any machine is online
            // then wake the server up
            if !self.server.machine.is_online && (self.always_on_state || any_machine_is_online) {
                info!("waking up {}...", self.server);
                match self.wakeup_server.wakeup() {
                    Err(_) => error!("failed to wake up {}", self.server),
                    Ok(_) => {
                        self.last_change = Instant::now();
                    }
                }
            } else if !self.always_on_state
                && !any_machine_is_online
                && self.server.machine.is_online
            {
                info!("shutting down {}...", self.server);
                match self.shutdown_server.shutdown() {
                    Err(e) => error!("failed to shut down {}: {}", self.server, e),
                    Ok(_) => {
                        self.last_change = Instant::now();
                    }
                }
            }
        }
    }

    fn update_machine_online(machine: &mut Machine, is_online: bool) {
        let machine_was_online = machine.is_online;

        // update the machines online state
        //   either if it is currently online
        //   or if it has become offline
        if is_online {
            debug!("received ping response from {}", machine);
            machine.set_online(true)
        } else {
            debug!("no ping response received from {}", machine);

            if machine_was_online
                && machine.last_seen.unwrap().elapsed()
                    > Duration::from_secs(machine.last_seen_timeout)
            {
                machine.set_online(false)
            }
        }

        if is_online != machine_was_online {
            if is_online {
                info!("{} is now online", machine);
            } else {
                info!("{} is now offline", machine);
            }
        }
    }
}
