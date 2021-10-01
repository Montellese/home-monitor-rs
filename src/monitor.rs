use std::ops::Sub;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, info, trace, warn};

use super::dom::{communication, Device, Machine, Server};
use super::networking::{Pinger, ShutdownServer, WakeupServer};
use super::utils::{AlwaysOff, AlwaysOn, Instant};

const CHANGE_TIMEOUT: Duration = Duration::from_secs(120);

pub struct Monitor {
    sender: Box<dyn communication::Sender>,

    server: Server,
    machines: Vec<Machine>,

    wakeup_server: Arc<dyn WakeupServer>,
    shutdown_server: Arc<dyn ShutdownServer>,

    always_off_state: bool,
    always_off: Arc<dyn AlwaysOff>,
    always_on_state: bool,
    always_on: Arc<dyn AlwaysOn>,

    last_ping: Instant,
    last_change: Instant,
    ping_interval: Duration,

    pinger: Box<dyn Pinger>,
}

impl Monitor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sender: Box<dyn communication::Sender>,
        ping_interval: Duration,
        server: Server,
        machines: Vec<Machine>,
        wakeup_server: Arc<dyn WakeupServer>,
        shutdown_server: Arc<dyn ShutdownServer>,
        pinger: Box<dyn Pinger>,
        always_off: Arc<dyn AlwaysOff>,
        always_on: Arc<dyn AlwaysOn>,
    ) -> Self {
        // get a mutable binding to pinger
        let mut mut_pinger = pinger;

        // add the IP address of the server to the pinger
        match mut_pinger.add_target(server.machine.ip) {
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
            return match mut_pinger.add_target(machine.ip) {
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

        assert!(
            !mut_machines.is_empty(),
            "no machines to monitor {}",
            server
        );

        // send the initial state of the server and all machines
        Self::publish_machine_update(&*sender, Device::Server(server.clone()));
        for machine in mut_machines.iter() {
            Self::publish_machine_update(&*sender, Device::Machine(machine.clone()));
        }

        let now = Instant::now();
        let last_ping = now.sub(ping_interval);
        let last_change = now.sub(CHANGE_TIMEOUT);

        Self {
            sender,
            server,
            machines: mut_machines,
            wakeup_server,
            shutdown_server,
            always_off_state: false,
            always_off,
            always_on_state: false,
            always_on,
            last_ping,
            last_change,
            ping_interval,
            pinger: mut_pinger,
        }
    }

    pub fn run_once(&mut self) {
        {
            // check the always off file
            let always_off_file_exists = self.always_off.is_always_off();
            // check the always on file
            let always_on_file_exists = self.always_on.is_always_on();

            // make sure we don't have always off and on simultaneously
            if always_off_file_exists && always_on_file_exists {
                warn!("ignoring ALWAYS OFF and ALWAYS ON because they are enabled simultaneously");
                self.always_off_state = false;
                self.always_on_state = false;
            } else if always_off_file_exists != self.always_off_state {
                if always_off_file_exists {
                    info!("ALWAYS OFF has been enabled");
                } else {
                    info!("ALWAYS OFF has been disabled");
                }

                self.always_off_state = always_off_file_exists;
            } else if always_on_file_exists != self.always_on_state {
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
                if Self::update_machine_online(&mut self.server.machine, server_is_online) {
                    Self::publish_machine_update(
                        &*self.sender,
                        Device::Server(self.server.clone()),
                    );
                }
            }

            // update the online state of all machines
            for machine in self.machines.iter_mut() {
                let is_online = self.pinger.is_online(&machine.ip);
                if Self::update_machine_online(machine, is_online) {
                    Self::publish_machine_update(&*self.sender, Device::Machine(machine.clone()));
                }
            }
        }

        // check if any machine is online
        let any_machine_is_online = self.machines.iter().any(|machine| machine.is_online);

        // process the collected information
        if self.always_off_state
            || self.always_on_state
            || self.last_change.elapsed() > CHANGE_TIMEOUT
        {
            // if the server is not online and
            //   the always on file exists or
            //   any machine is online
            // then wake the server up
            if !self.server.machine.is_online
                && !self.always_off_state
                && (self.always_on_state || any_machine_is_online)
            {
                info!("waking up {}...", self.server);
                match self.wakeup_server.wakeup() {
                    Err(_) => error!("failed to wake up {}", self.server),
                    Ok(_) => {
                        self.last_change = Instant::now();
                    }
                }
            } else if self.server.machine.is_online
                && !self.always_on_state
                && (self.always_off_state || !any_machine_is_online)
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

    fn update_machine_online(machine: &mut Machine, is_online: bool) -> bool {
        let machine_was_online = machine.is_online;

        // update the machines online state
        //   either if it is currently online
        //   or if it has become offline
        if is_online {
            trace!("received ping response from {}", machine);
            machine.set_online(true)
        } else {
            trace!("no ping response received from {}", machine);

            if machine_was_online
                && machine.last_seen.unwrap().elapsed()
                    > Duration::from_secs(machine.last_seen_timeout)
            {
                machine.set_online(false)
            }
        }

        if machine.is_online != machine_was_online {
            if machine.is_online {
                info!("{} is now online", machine);
            } else {
                info!("{} is now offline", machine);
            }

            return true;
        }

        false
    }

    fn publish_machine_update(sender: &dyn communication::Sender, device: Device) {
        debug!("publishing update for {}", device);
        if let Err(e) = sender.send(device.clone()) {
            warn!("failed to publish update for {}: {}", device, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::net::IpAddr;
    use std::ops::Add;
    use std::sync::mpsc::RecvError;

    use mockall::predicate::*;
    use mockall::Sequence;
    use rstest::*;

    use super::*;

    static PING_INTERVAL: Duration = Duration::from_secs(1);

    static SERVER_NAME: &str = "Test Server";
    static SERVER_MAC: &str = "aa:bb:cc:dd:ee:ff";
    static SERVER_IP: &str = "10.0.0.1";
    const SERVER_LAST_SEEN_TIMEOUT: u64 = 60;
    static SERVER_USERNAME: &str = "username";
    static SERVER_PASSWORD: &str = "password";

    static MACHINE_NAME: &str = "Test Machine";
    static MACHINE_IP: &str = "10.0.0.2";
    const MACHINE_LAST_SEEN_TIMEOUT: u64 = 300;

    #[fixture]
    fn fake_clock() {
        let mut max_duration: Duration = std::cmp::max(
            CHANGE_TIMEOUT,
            Duration::from_secs(MACHINE_LAST_SEEN_TIMEOUT),
        );
        max_duration = max_duration.add(Duration::from_secs(1));
        Instant::set_time(max_duration.as_millis().try_into().unwrap());
    }

    #[fixture]
    fn server_ip() -> IpAddr {
        SERVER_IP.parse().unwrap()
    }

    #[fixture]
    fn server() -> Server {
        Server::new(
            SERVER_NAME,
            server_ip(),
            SERVER_LAST_SEEN_TIMEOUT,
            SERVER_MAC,
            SERVER_USERNAME,
            SERVER_PASSWORD,
        )
    }

    #[fixture]
    fn machine_ip() -> IpAddr {
        MACHINE_IP.parse().unwrap()
    }

    #[fixture]
    fn machine() -> Machine {
        Machine::new(MACHINE_NAME, machine_ip(), MACHINE_LAST_SEEN_TIMEOUT)
    }

    fn default_mocks() -> (
        Box<crate::dom::communication::MockSender>,
        crate::networking::MockWakeupServer,
        crate::networking::MockShutdownServer,
        Box<crate::networking::MockPinger>,
        crate::utils::MockAlwaysOff,
        crate::utils::MockAlwaysOn,
    ) {
        (
            Box::new(crate::dom::communication::MockSender::new()),
            crate::networking::MockWakeupServer::new(),
            crate::networking::MockShutdownServer::new(),
            Box::new(crate::networking::MockPinger::new()),
            crate::utils::MockAlwaysOff::new(),
            crate::utils::MockAlwaysOn::new(),
        )
    }

    #[rstest]
    #[should_panic(expected = "no machines to monitor")]
    #[allow(unused_variables)]
    fn test_monitor_fails_without_machines(fake_clock: (), server_ip: IpAddr, server: Server) {
        // SETUP
        let (sender, wakeup_server, shutdown_server, mut pinger, always_off, always_on) =
            default_mocks();

        let machines = vec![];

        // EXPECTATIONS
        pinger
            .expect_add_target()
            .with(eq(server_ip))
            .once()
            .returning(|_| Ok(true));

        // TESTING
        #[allow(unused_variables)]
        let monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_ignore_duplicate_machine_ips(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (mut sender, wakeup_server, shutdown_server, mut pinger, always_off, always_on) =
            default_mocks();

        let machines = vec![
            machine,
            Machine::new("Test Machine 2", machine_ip, MACHINE_LAST_SEEN_TIMEOUT),
        ];

        // EXPECTATIONS
        let mut seq = Sequence::new();
        pinger
            .expect_add_target()
            .with(eq(server_ip))
            .once()
            .return_once(|_| Ok(true))
            .in_sequence(&mut seq);
        pinger
            .expect_add_target()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| Ok(true))
            .in_sequence(&mut seq);
        pinger
            .expect_add_target()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| Ok(false))
            .in_sequence(&mut seq);
        sender
            .expect_send()
            .times(2)
            .returning(|_| Ok(()))
            .in_sequence(&mut seq);

        // TESTING
        #[allow(unused_variables)]
        let monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_always_off_and_on_checked_in_run_once(
        fake_clock: (),
        server: Server,
        machine: Machine,
    ) {
        // SETUP
        let (mut sender, wakeup_server, shutdown_server, mut pinger, mut always_off, mut always_on) =
            default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        // ping_once() is not called as long as the ping interval hasn't expired
        pinger.expect_ping_once().never();

        // is_always_off() is always called
        always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);

        // is_always_on() is always called
        always_on.expect_is_always_on().once().return_once(|| false);

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_ignore_if_always_off_and_on(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (
            mut sender,
            mut wakeup_server,
            mut shutdown_server,
            mut pinger,
            mut always_off,
            mut always_on,
        ) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        always_off
            .expect_is_always_off()
            .once()
            .return_once(|| true);
        always_on.expect_is_always_on().once().return_once(|| true);

        wakeup_server.expect_wakeup().never();
        shutdown_server.expect_shutdown().never();

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_shutdown_server_if_always_off(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (
            mut sender,
            wakeup_server,
            mut shutdown_server,
            mut pinger,
            mut always_off,
            mut always_on,
        ) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        always_off
            .expect_is_always_off()
            .once()
            .return_once(|| true);
        always_on.expect_is_always_on().once().return_once(|| false);

        {
            // we need to simulate that the server and machine are online
            let mut seq = Sequence::new();
            pinger
                .expect_ping_once()
                .once()
                .return_once(|| {})
                .in_sequence(&mut seq);
            pinger
                .expect_recv_pong()
                .once()
                .return_once(|| Ok(()))
                .in_sequence(&mut seq);
            pinger
                .expect_is_online()
                .with(eq(server_ip))
                .once()
                .return_once(|_| true)
                .in_sequence(&mut seq);
            sender
                .expect_send()
                .once()
                .return_once(|_| Ok(()))
                .in_sequence(&mut seq);
            pinger
                .expect_is_online()
                .with(eq(machine_ip))
                .once()
                .return_once(|_| true)
                .in_sequence(&mut seq);
            sender
                .expect_send()
                .once()
                .return_once(|_| Ok(()))
                .in_sequence(&mut seq);
        }

        shutdown_server
            .expect_shutdown()
            .once()
            .return_once(|| Ok(()));

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_wakeup_server_if_always_on(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (
            mut sender,
            mut wakeup_server,
            shutdown_server,
            mut pinger,
            mut always_off,
            mut always_on,
        ) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        always_on.expect_is_always_on().once().return_once(|| true);

        wakeup_server.expect_wakeup().once().return_once(|| Ok(()));

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_ping_once_if_interval_elapsed(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (mut sender, wakeup_server, shutdown_server, mut pinger, mut always_off, mut always_on) =
            default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        always_on.expect_is_always_on().once().return_once(|| false);

        let mut seq = Sequence::new();
        pinger
            .expect_ping_once()
            .once()
            .return_once(|| {})
            .in_sequence(&mut seq);
        pinger
            .expect_recv_pong()
            .once()
            .return_once(|| Ok(()))
            .in_sequence(&mut seq);
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .once()
            .return_once(|_| false)
            .in_sequence(&mut seq);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| false)
            .in_sequence(&mut seq);

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        monitor.run_once();
    }

    #[rstest]
    #[should_panic(expected = "Pinger failed to receive responses")]
    #[allow(unused_variables)]
    fn test_monitor_fails_if_recv_pong_fails(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (mut sender, wakeup_server, shutdown_server, mut pinger, mut always_off, mut always_on) =
            default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        always_on.expect_is_always_on().once().return_once(|| false);

        let mut seq = Sequence::new();
        pinger
            .expect_ping_once()
            .once()
            .return_once(|| {})
            .in_sequence(&mut seq);
        pinger
            .expect_recv_pong()
            .once()
            .return_once(|| Err(RecvError))
            .in_sequence(&mut seq);

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_wakeup_server_if_at_least_one_machine_is_online(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (
            mut sender,
            mut wakeup_server,
            shutdown_server,
            mut pinger,
            mut always_off,
            mut always_on,
        ) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        always_on.expect_is_always_on().once().return_once(|| false);

        let mut seq = Sequence::new();
        pinger
            .expect_ping_once()
            .once()
            .return_once(|| {})
            .in_sequence(&mut seq);
        pinger
            .expect_recv_pong()
            .once()
            .return_once(|| Ok(()))
            .in_sequence(&mut seq);
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .once()
            .return_once(|_| false)
            .in_sequence(&mut seq);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| true)
            .in_sequence(&mut seq);
        sender
            .expect_send()
            .once()
            .return_once(|_| Ok(()))
            .in_sequence(&mut seq);

        wakeup_server.expect_wakeup().once().return_once(|| Ok(()));

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_only_wakeup_server_again_if_change_timeout_expired(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (
            mut sender,
            mut wakeup_server,
            shutdown_server,
            mut pinger,
            mut always_off,
            mut always_on,
        ) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        always_off.expect_is_always_off().returning(|| false);
        always_on.expect_is_always_on().returning(|| false);

        pinger.expect_ping_once().returning(|| {});
        pinger.expect_recv_pong().returning(|| Ok(()));
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .returning(|_| false);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .returning(|_| true);
        sender.expect_send().once().return_once(|_| Ok(()));

        wakeup_server.expect_wakeup().times(2).returning(|| Ok(()));

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        monitor.run_once();

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        // this run should not wakeup the server
        monitor.run_once();

        // advance FakeClock by at least change timeout (120s)
        Instant::advance_time((2 * CHANGE_TIMEOUT).as_millis().try_into().unwrap());

        // this run should wakeup the server again
        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_shutdown_server_if_no_machine_is_online(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (
            mut sender,
            wakeup_server,
            mut shutdown_server,
            mut pinger,
            mut always_off,
            mut always_on,
        ) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        always_on.expect_is_always_on().returning(|| false);

        pinger.expect_ping_once().returning(|| {});
        pinger.expect_recv_pong().returning(|| Ok(()));
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .returning(|_| true);
        sender.expect_send().once().return_once(|_| Ok(()));
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .returning(|_| false);

        shutdown_server
            .expect_shutdown()
            .once()
            .return_once(|| Ok(()));

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_only_shutdown_server_after_wakeup_if_change_timeout_expired(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (
            mut sender,
            mut wakeup_server,
            mut shutdown_server,
            mut pinger,
            mut always_off,
            mut always_on,
        ) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        let mut seq = Sequence::new();

        // first call to ping_once() which will wakeup the server
        always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false)
            .in_sequence(&mut seq);
        always_on
            .expect_is_always_on()
            .once()
            .return_once(|| false)
            .in_sequence(&mut seq);
        pinger
            .expect_ping_once()
            .once()
            .return_once(|| {})
            .in_sequence(&mut seq);
        pinger
            .expect_recv_pong()
            .once()
            .return_once(|| Ok(()))
            .in_sequence(&mut seq);
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .once()
            .return_once(|_| false)
            .in_sequence(&mut seq);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| true)
            .in_sequence(&mut seq);
        sender
            .expect_send()
            .once()
            .return_once(|_| Ok(()))
            .in_sequence(&mut seq);
        wakeup_server
            .expect_wakeup()
            .once()
            .return_once(|| Ok(()))
            .in_sequence(&mut seq);

        // second call to ping_once() which will not shutdown the server
        always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false)
            .in_sequence(&mut seq);
        always_on
            .expect_is_always_on()
            .once()
            .return_once(|| false)
            .in_sequence(&mut seq);
        pinger
            .expect_ping_once()
            .once()
            .return_once(|| {})
            .in_sequence(&mut seq);
        pinger
            .expect_recv_pong()
            .once()
            .return_once(|| Ok(()))
            .in_sequence(&mut seq);
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .once()
            .return_once(|_| true)
            .in_sequence(&mut seq);
        sender
            .expect_send()
            .once()
            .return_once(|_| Ok(()))
            .in_sequence(&mut seq);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| false)
            .in_sequence(&mut seq);

        // third call to ping_once() which will shutdown the server
        always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false)
            .in_sequence(&mut seq);
        always_on
            .expect_is_always_on()
            .once()
            .return_once(|| false)
            .in_sequence(&mut seq);
        pinger
            .expect_ping_once()
            .once()
            .return_once(|| {})
            .in_sequence(&mut seq);
        pinger
            .expect_recv_pong()
            .once()
            .return_once(|| Ok(()))
            .in_sequence(&mut seq);
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .once()
            .return_once(|_| true)
            .in_sequence(&mut seq);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| false)
            .in_sequence(&mut seq);
        sender
            .expect_send()
            .once()
            .return_once(|_| Ok(()))
            .in_sequence(&mut seq);
        shutdown_server
            .expect_shutdown()
            .once()
            .return_once(|| Ok(()))
            .in_sequence(&mut seq);

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        monitor.run_once();

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        // this run should not shutdown the server
        monitor.run_once();

        // advance FakeClock by at least change timeout (120s) or last seen timeout (300s)
        let max_timeout = std::cmp::max(
            Duration::from_secs(MACHINE_LAST_SEEN_TIMEOUT),
            CHANGE_TIMEOUT,
        );
        Instant::advance_time((2 * max_timeout).as_millis().try_into().unwrap());

        // this run should shutdown the server
        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_dont_wakeup_server_if_always_off(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (
            mut sender,
            mut wakeup_server,
            shutdown_server,
            mut pinger,
            mut always_off,
            mut always_on,
        ) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        always_off.expect_is_always_off().returning(|| true);

        always_on.expect_is_always_on().returning(|| false);

        pinger.expect_ping_once().returning(|| {});
        pinger.expect_recv_pong().returning(|| Ok(()));
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .returning(|_| false);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .returning(|_| true);
        sender.expect_send().once().return_once(|_| Ok(()));

        wakeup_server.expect_wakeup().never();

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_dont_shutdown_server_if_always_on(
        fake_clock: (),
        server_ip: IpAddr,
        server: Server,
        machine_ip: IpAddr,
        machine: Machine,
    ) {
        // SETUP
        let (
            mut sender,
            wakeup_server,
            mut shutdown_server,
            mut pinger,
            mut always_off,
            mut always_on,
        ) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| Ok(true));
        sender.expect_send().times(2).returning(|_| Ok(()));

        always_off.expect_is_always_off().returning(|| false);

        always_on.expect_is_always_on().returning(|| true);

        pinger.expect_ping_once().returning(|| {});
        pinger.expect_recv_pong().returning(|| Ok(()));
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .returning(|_| true);
        sender.expect_send().once().return_once(|_| Ok(()));
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .returning(|_| false);

        shutdown_server.expect_shutdown().never();

        // TESTING
        #[allow(unused_variables)]
        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            server,
            machines,
            Arc::new(wakeup_server),
            Arc::new(shutdown_server),
            pinger,
            Arc::new(always_off),
            Arc::new(always_on),
        );

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        monitor.run_once();
    }
}
