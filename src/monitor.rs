use std::collections::HashMap;
use std::ops::Sub;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use log::{debug, error, info, trace, warn};

use super::control::ServerControl;
use super::dom::{communication, Dependencies, Device, DeviceId, Machine, Server};
use super::networking::Pinger;
use super::utils::Instant;

const CHANGE_TIMEOUT: Duration = Duration::from_secs(120);

type SharedDevice = Arc<RwLock<Device>>;

struct MonitoredServer {
    pub control: ServerControl,
    pub server: SharedDevice,
    pub devices: Vec<SharedDevice>,
    pub always_off_state: bool,
    pub always_on_state: bool,
    pub last_change: Instant,
}

impl MonitoredServer {
    pub fn new(
        control: ServerControl,
        server: SharedDevice,
        devices: Vec<SharedDevice>,
        last_change: Instant,
    ) -> Self {
        Self {
            control,
            server,
            devices,
            always_off_state: false,
            always_on_state: false,
            last_change,
        }
    }

    pub fn server(&self) -> &Server {
        &self.control.server
    }

    pub fn process(&mut self) {
        trace!("processing {}...", self.server());

        // first update the internal state of the files API
        self.update_files_api();

        // check if any device is online
        let any_device_is_online = self
            .devices
            .iter()
            .any(|device| device.read().unwrap().is_online());

        // process the collected information
        if self.always_off_state
            || self.always_on_state
            || self.last_change.elapsed() > CHANGE_TIMEOUT
        {
            let server = self.server.read().unwrap();

            // if the server is not online and
            //   the always on file exists or
            //   any device is online
            // then wake the server up
            if !server.is_online()
                && !self.always_off_state
                && (self.always_on_state || any_device_is_online)
            {
                info!("waking up {}...", server);
                match self.control.wakeup.wakeup() {
                    Err(_) => error!("failed to wake up {}", server),
                    Ok(_) => {
                        self.last_change = Instant::now();
                    }
                }
            } else if server.is_online()
                && !self.always_on_state
                && (self.always_off_state || !any_device_is_online)
            {
                info!("shutting down {}...", server);
                match self.control.shutdown.shutdown() {
                    Err(e) => error!("failed to shut down {}: {}", server, e),
                    Ok(_) => {
                        self.last_change = Instant::now();
                    }
                }
            }
        }
    }

    fn update_files_api(&mut self) {
        // check the always off file
        let always_off_file_exists = self.control.always_off.is_always_off();
        // check the always on file
        let always_on_file_exists = self.control.always_on.is_always_on();

        // make sure we don't have always off and on simultaneously
        if always_off_file_exists && always_on_file_exists {
            warn!(
                "{}: ignoring ALWAYS OFF and ALWAYS ON because they are enabled simultaneously",
                self.server()
            );
            self.always_off_state = false;
            self.always_on_state = false;
        } else if always_off_file_exists != self.always_off_state {
            if always_off_file_exists {
                info!("{}: ALWAYS OFF has been enabled", self.server());
            } else {
                info!("{}: ALWAYS OFF has been disabled", self.server());
            }

            self.always_off_state = always_off_file_exists;
        } else if always_on_file_exists != self.always_on_state {
            if always_on_file_exists {
                info!("{}: ALWAYS ON has been enabled", self.server());
            } else {
                info!("{}: ALWAYS ON has been disabled", self.server());
            }

            self.always_on_state = always_on_file_exists;
        }
    }
}

pub struct Monitor {
    sender: Box<dyn communication::Sender>,

    servers: Vec<MonitoredServer>,
    devices: Vec<SharedDevice>,

    last_ping: Instant,
    ping_interval: Duration,

    pinger: Box<dyn Pinger>,
}

impl Monitor {
    pub fn new(
        sender: Box<dyn communication::Sender>,
        ping_interval: Duration,
        server_controls: Vec<ServerControl>,
        machines: Vec<Machine>,
        dependencies: Dependencies,
        pinger: Box<dyn Pinger>,
    ) -> Self {
        assert!(!machines.is_empty(), "no machines to monitor");

        // collect all monitored machines into a hashmap
        let mut monitored_devices: HashMap<DeviceId, SharedDevice> = machines
            .into_iter()
            .map(|machine| {
                (
                    machine.id.clone(),
                    Arc::new(RwLock::new(Device::Machine(machine))),
                )
            })
            .collect();
        // extend the monitored devices with the controlled servers (which are also monitored)
        monitored_devices.extend(server_controls.iter().map(|control| {
            (
                control.server.machine.id.clone(),
                Arc::new(RwLock::new(Device::Server(control.server.clone()))),
            )
        }));

        // get a mutable binding to pinger
        let mut mut_pinger = pinger;

        // add the IP addresses of all devices to the pinger
        for (_, device) in monitored_devices.iter() {
            let result = match &*device.write().unwrap() {
                Device::Server(server) => mut_pinger.add_target(server.machine.ip),
                Device::Machine(machine) => mut_pinger.add_target(machine.ip),
            };

            assert!(
                result,
                "failed to add {} to the pinger",
                device.read().unwrap()
            );
        }

        // send the initial state of all devices
        for (_, device) in monitored_devices.iter() {
            Self::publish_device_update(&*sender, device.read().unwrap().clone());
        }

        let now = Instant::now();
        let last_ping = now.sub(ping_interval);
        let last_change = now.sub(CHANGE_TIMEOUT);

        let mut servers = Vec::new();
        for control in server_controls {
            // get the monitored device matching the controlled server
            let server = monitored_devices
                .get(&control.server.machine.id)
                .unwrap()
                .clone();

            // get all dependencies (as a list of device IDs) of the server to control
            let deps = dependencies.get(&control.server.machine.id).unwrap();

            // get weak references to all the devices
            let devices = deps
                .iter()
                .map(|device_id| monitored_devices.get(device_id).unwrap().clone())
                .collect();

            servers.push(MonitoredServer::new(control, server, devices, last_change));
        }

        Self {
            sender,
            servers,
            devices: monitored_devices
                .into_iter()
                .map(|(_, device)| device)
                .collect(),
            last_ping,
            ping_interval,
            pinger: mut_pinger,
        }
    }

    pub fn run_once(&mut self) {
        // check if the devices are online
        if self.last_ping.elapsed() > self.ping_interval {
            self.last_ping = Instant::now();

            // determine the number of machines (+ server)
            let num_devices = self.devices.len();

            // run the pinger once
            debug!("pinging {} devices...", num_devices);
            self.pinger.ping_once();
            // and receive all responses (pongs)
            if let Err(e) = self.pinger.recv_pong() {
                panic!("Pinger failed to receive responses: {}", e)
            }

            // update the online state of all devices
            for device in self.devices.iter_mut() {
                trace!("updating online state of {}...", device.read().unwrap());
                let is_device_online = self.pinger.is_online(device.read().unwrap().ip());
                if Self::update_device_online(&mut device.write().unwrap(), is_device_online) {
                    Self::publish_device_update(&*self.sender, device.read().unwrap().clone());
                }
            }
        }

        // go through all controlled servers
        for server in self.servers.iter_mut() {
            server.process();
        }
    }

    fn update_device_online(device: &mut Device, is_online: bool) -> bool {
        let device_was_online = device.is_online();

        // update the machines online state
        //   either if it is currently online
        //   or if it has become offline
        if is_online {
            trace!("received ping response from {}", device);
            device.set_online(true)
        } else {
            trace!("no ping response received from {}", device);

            if device_was_online
                && device.last_seen().unwrap().elapsed()
                    > Duration::from_secs(device.last_seen_timeout())
            {
                device.set_online(false)
            }
        }

        let device_is_online = device.is_online();
        if device_is_online != device_was_online {
            if device_is_online {
                info!("{} is now online", device);
            } else {
                info!("{} is now offline", device);
            }

            return true;
        }

        false
    }

    fn publish_device_update(sender: &dyn communication::Sender, device: Device) {
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
    use crate::control::test::*;
    use crate::dom::device::test::*;
    use crate::dom::test::*;

    static PING_INTERVAL: Duration = Duration::from_secs(1);

    #[fixture]
    fn fake_clock() {
        let mut max_duration: Duration = std::cmp::max(
            CHANGE_TIMEOUT,
            Duration::from_secs(MACHINE_LAST_SEEN_TIMEOUT),
        );
        max_duration = max_duration.add(Duration::from_secs(1));
        Instant::set_time(max_duration.as_millis().try_into().unwrap());
    }

    fn default_mocks() -> (
        Box<crate::dom::communication::MockSender>,
        Box<crate::networking::MockPinger>,
    ) {
        (
            Box::new(crate::dom::communication::MockSender::new()),
            Box::new(crate::networking::MockPinger::new()),
        )
    }

    #[rstest]
    #[should_panic(expected = "no machines to monitor")]
    #[allow(unused_variables)]
    fn test_monitor_fails_without_machines(
        fake_clock: (),
        server_ip: IpAddr,
        mocked_server_control: MockServerControl,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (sender, mut pinger) = default_mocks();

        let servers = vec![ServerControl::from(mocked_server_control)];
        let machines = vec![];

        // EXPECTATIONS
        pinger
            .expect_add_target()
            .with(eq(server_ip))
            .once()
            .returning(|_| true);

        // TESTING
        #[allow(unused_variables)]
        let monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
        );
    }

    #[rstest]
    #[should_panic(expected = "failed to add")]
    #[allow(unused_variables)]
    fn test_monitor_fails_on_duplicate_ips(
        fake_clock: (),
        server_ip: IpAddr,
        mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (sender, mut pinger) = default_mocks();

        let servers = vec![ServerControl::from(mocked_server_control)];
        let machines = vec![
            machine,
            Machine::new(
                &"testmachine2".parse().unwrap(),
                "Test Machine 2",
                machine_ip,
                MACHINE_LAST_SEEN_TIMEOUT,
            ),
        ];

        // EXPECTATIONS
        pinger
            .expect_add_target()
            .with(eq(server_ip))
            .once()
            .return_once(|_| true);
        pinger
            .expect_add_target()
            .with(eq(machine_ip))
            .times(2)
            .return_once(|_| true)
            .return_once(|_| false);

        // TESTING
        #[allow(unused_variables)]
        let monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
        );
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_always_off_and_on_checked_in_run_once(
        fake_clock: (),
        mut mocked_server_control: MockServerControl,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        // ping_once() is not called as long as the ping interval hasn't expired
        pinger.expect_ping_once().never();

        // is_always_off() is always called
        mocked_server_control
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);

        // is_always_on() is always called
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .once()
            .return_once(|| false);

        // TESTING
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
        );

        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_ignore_if_always_off_and_on(
        fake_clock: (),
        server_ip: IpAddr,
        mut mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        mocked_server_control
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| true);
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .once()
            .return_once(|| true);

        mocked_server_control.wakeup.expect_wakeup().never();
        mocked_server_control.shutdown.expect_shutdown().never();

        // TESTING
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
        );

        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_shutdown_server_if_always_off(
        fake_clock: (),
        server_ip: IpAddr,
        mut mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        mocked_server_control
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| true);
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .once()
            .return_once(|| false);

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
        }

        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .once()
            .return_once(|_| true);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        mocked_server_control
            .shutdown
            .expect_shutdown()
            .once()
            .return_once(|| Ok(()));

        // TESTING
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
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
        mut mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        mocked_server_control
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .once()
            .return_once(|| true);

        mocked_server_control
            .wakeup
            .expect_wakeup()
            .once()
            .return_once(|| Ok(()));

        // TESTING
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
        );

        monitor.run_once();
    }

    #[rstest]
    #[allow(unused_variables)]
    fn test_monitor_ping_once_if_interval_elapsed(
        fake_clock: (),
        server_ip: IpAddr,
        mut mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        mocked_server_control
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .once()
            .return_once(|| false);

        {
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
        }
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .once()
            .return_once(|_| false);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| false);

        // TESTING
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
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
        mut mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        mocked_server_control
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .once()
            .return_once(|| false);

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
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
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
        mut mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        mocked_server_control
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .once()
            .return_once(|| false);

        {
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
        }

        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .once()
            .return_once(|_| false);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| true);
        sender.expect_send().once().return_once(|_| Ok(()));

        mocked_server_control
            .wakeup
            .expect_wakeup()
            .once()
            .return_once(|| Ok(()));

        // TESTING
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
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
        mut mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        mocked_server_control
            .always_off
            .expect_is_always_off()
            .returning(|| false);
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .returning(|| false);

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

        mocked_server_control
            .wakeup
            .expect_wakeup()
            .times(2)
            .returning(|| Ok(()));

        // TESTING
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
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
        mut mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        mocked_server_control
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .returning(|| false);

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

        mocked_server_control
            .shutdown
            .expect_shutdown()
            .once()
            .return_once(|| Ok(()));

        // TESTING
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
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
        mut mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        // first call to ping_once() which will wakeup the server
        mocked_server_control
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .once()
            .return_once(|| false);
        pinger.expect_ping_once().once().return_once(|| {});
        pinger.expect_recv_pong().once().return_once(|| Ok(()));
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .once()
            .return_once(|_| false);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| true);
        sender.expect_send().once().return_once(|_| Ok(()));
        mocked_server_control
            .wakeup
            .expect_wakeup()
            .once()
            .return_once(|| Ok(()));

        // second call to ping_once() which will not shutdown the server
        mocked_server_control
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .once()
            .return_once(|| false);
        pinger.expect_ping_once().once().return_once(|| {});
        pinger.expect_recv_pong().once().return_once(|| Ok(()));
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .once()
            .return_once(|_| true);
        sender.expect_send().once().return_once(|_| Ok(()));
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| false);

        // third call to ping_once() which will shutdown the server
        mocked_server_control
            .always_off
            .expect_is_always_off()
            .once()
            .return_once(|| false);
        mocked_server_control
            .always_on
            .expect_is_always_on()
            .once()
            .return_once(|| false);
        pinger.expect_ping_once().once().return_once(|| {});
        pinger.expect_recv_pong().once().return_once(|| Ok(()));
        pinger
            .expect_is_online()
            .with(eq(server_ip))
            .once()
            .return_once(|_| true);
        pinger
            .expect_is_online()
            .with(eq(machine_ip))
            .once()
            .return_once(|_| false);
        sender.expect_send().once().return_once(|_| Ok(()));
        mocked_server_control
            .shutdown
            .expect_shutdown()
            .once()
            .return_once(|| Ok(()));

        // TESTING
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
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
        mut mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        mocked_server_control
            .always_off
            .expect_is_always_off()
            .returning(|| true);

        mocked_server_control
            .always_on
            .expect_is_always_on()
            .returning(|| false);

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

        mocked_server_control.wakeup.expect_wakeup().never();

        // TESTING
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
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
        mut mocked_server_control: MockServerControl,
        machine_ip: IpAddr,
        machine: Machine,
        dependencies: Dependencies,
    ) {
        // SETUP
        let (mut sender, mut pinger) = default_mocks();

        let machines = vec![machine];

        // EXPECTATIONS
        pinger.expect_add_target().returning(|_| true);
        sender.expect_send().times(2).returning(|_| Ok(()));

        mocked_server_control
            .always_off
            .expect_is_always_off()
            .returning(|| false);

        mocked_server_control
            .always_on
            .expect_is_always_on()
            .returning(|| true);

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

        mocked_server_control.shutdown.expect_shutdown().never();

        // TESTING
        let servers = vec![ServerControl::from(mocked_server_control)];

        let mut monitor = Monitor::new(
            sender,
            PING_INTERVAL,
            servers,
            machines,
            dependencies,
            pinger,
        );

        // advance FakeClock by at least ping interval (1s)
        Instant::advance_time((2 * PING_INTERVAL).as_millis().try_into().unwrap());

        monitor.run_once();
    }
}
