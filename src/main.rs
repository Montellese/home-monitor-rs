use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser;
use log::{debug, error, info, warn};
use simplelog::{LevelFilter, SimpleLogger};

use crate::networking::{PortChecker, TcpPortChecker};

mod configuration;
mod control;
mod dom;
mod env;
mod monitor;
mod networking;
mod utils;
mod web;

#[derive(Parser)]
#[clap(author, version, about)]
struct Opts {
    // Path to the JSON configuration file
    #[clap(
        short = 'c',
        long = "config",
        value_name = "FILE",
        default_value = configuration::LOCATION
    )]
    config: String,

    // Enable debug logging
    #[clap(short = 'd', long = "debug", group = "verbosity")]
    debug: bool,

    // Enable verbose logging
    #[clap(
        short = 'v',
        long = "verbose",
        conflicts_with = "debug",
        group = "verbosity"
    )]
    verbose: bool,

    // Shut down the specified server(s)
    #[clap(
        short = 's',
        long = "shutdown",
        num_args = 1..,
        value_name = "SERVER",
        conflicts_with_all = ["wakeup", "wait_online"],
        group = "mode"
    )]
    shutdown: Vec<String>,

    // Wake up the specified server(s)
    #[clap(
        short = 'w',
        long = "wakeup",
        num_args = 1..,
        value_name = "SERVER",
        conflicts_with_all = ["shutdown", "wait_online"],
        group = "mode"
    )]
    wakeup: Vec<String>,

    // Wait until the specified server(s) is/are online
    #[clap(
        short = 'n',
        long = "wait-online",
        num_args = 1..,
        value_name = "SERVER",
        conflicts_with_all = ["shutdown", "wakeup"],
        group = "mode"
    )]
    wait_online: Vec<String>,
}

enum Mode {
    Wakeup,
    Shutdown,
    WaitOnline,
}

fn run(
    args: Opts,
    config: configuration::Configuration,
    configured_servers: HashMap<configuration::DeviceId, configuration::Server>,
    configured_machines: HashMap<configuration::DeviceId, configuration::Machine>,
) -> exitcode::ExitCode {
    // check if a manual option has been provided
    if !args.wakeup.is_empty() || !args.shutdown.is_empty() || !args.wait_online.is_empty() {
        let mode: Mode;
        let servers: &Vec<String>;
        if !args.wakeup.is_empty() {
            mode = Mode::Wakeup;
            servers = &args.wakeup;
        } else if !args.shutdown.is_empty() {
            mode = Mode::Shutdown;
            servers = &args.shutdown;
        } else {
            mode = Mode::WaitOnline;
            servers = &args.wait_online;
        }

        // make sure all provided servers are also configured
        if !servers
            .iter()
            .all(|server_id| configured_servers.contains_key(&server_id.parse().unwrap()))
        {
            error!("unconfigured server(s) provided");
            return exitcode::USAGE;
        }

        // process provided servers
        let mut exitcode = exitcode::OK;
        for server_id in servers {
            let configured_server = configured_servers.get(&server_id.parse().unwrap()).unwrap();
            let server = dom::Server::from(configured_server);

            match mode {
                Mode::Wakeup => {
                    info!("waking up {} ({})...", server.machine.name, server_id);
                    let wakeup_server = control::Factory::create_wakeup_server(&server);
                    match wakeup_server.wakeup() {
                        Err(_) => {
                            error!("failed to wake up {} ({})", server.machine.name, server_id);
                            exitcode = exitcode::UNAVAILABLE;
                        }
                        Ok(_) => info!(
                            "{} ({}) successfully woken up",
                            server.machine.name, server_id
                        ),
                    };
                }

                Mode::Shutdown => {
                    info!("shutting down {} ({})...", server.machine.name, server_id);
                    let shutdown_server = control::Factory::create_shutdown_server(&server);
                    match shutdown_server.shutdown() {
                        Err(e) => {
                            error!(
                                "failed to shut down {} ({}): {}",
                                server.machine.name, server_id, e
                            );
                            exitcode = exitcode::UNAVAILABLE;
                        }
                        Ok(_) => info!(
                            "{} ({}) successfully shut down",
                            server.machine.name, server_id
                        ),
                    };
                }

                Mode::WaitOnline => {
                    info!(
                        "waiting for {} ({}) to be online...",
                        server.machine.name, server_id
                    );

                    let timeout = server.machine.last_seen_timeout;
                    let tcp_port_checker = TcpPortChecker::new(
                        server.machine.ip,
                        server.ssh.port.into(),
                        Duration::from_secs(1),
                    );

                    exitcode = exitcode::UNAVAILABLE;
                    for secs in 0..timeout {
                        debug!(
                            "checking TCP port {} on {} ({})",
                            Into::<u16>::into(server.ssh.port),
                            server.machine.name,
                            server_id
                        );
                        if tcp_port_checker.check() {
                            info!(
                                "{} ({}) is online after {} seconds",
                                server.machine.name, server_id, secs
                            );
                            exitcode = exitcode::OK;
                            break;
                        }
                    }

                    if exitcode == exitcode::UNAVAILABLE {
                        warn!(
                            "{} ({}) is not online after {} seconds",
                            server.machine.name, server_id, timeout
                        );
                    }
                }
            }
        }

        exitcode
    } else {
        // make sure machines are configured
        if configured_machines.is_empty() {
            error!("configuration doesn't contain any machines to monitor");
            std::process::exit(exitcode::CONFIG);
        }

        // make sure dependencies are configured
        if config.dependencies.0.is_empty() {
            error!("no dependencies configured");
            std::process::exit(exitcode::CONFIG);
        }
        let ping_interval = Duration::from_secs(config.network.ping.interval);

        // create the server DOM objects from the parsed configuration
        let servers: Vec<dom::Server> =
            configured_servers.values().map(dom::Server::from).collect();

        // create the machine DOM objects from the parsed configuration
        let machines: Vec<dom::Machine> = configured_machines
            .values()
            .map(dom::Machine::from)
            .collect();

        process(args, config, ping_interval, servers, machines)
    }
}

fn process(
    args: Opts,
    config: configuration::Configuration,
    ping_interval: Duration,
    servers: Vec<dom::Server>,
    machines: Vec<dom::Machine>,
) -> exitcode::ExitCode {
    // create the tokio runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(web::Server::get_num_workers())
        .thread_name(web::Server::get_thread_name(env::PKG_NAME))
        .enable_all()
        .build()
        .expect("failed to build a tokio runtime");

    // setup SIGINT signal handling
    debug!("setting up signal handling for SIGTERM");
    let sigterm = tokio::signal::ctrl_c();

    // prepare a channel to communicate updates from monitoring to the web API
    let (tx, rx) = dom::communication::mpsc_channel();

    // only start the web API (and shared state synchronization) if a valid port is configured
    let provide_web_api = config.api.web.port > 0;

    // prepare the server controls
    let server_controls: Vec<control::ServerControl> = servers
        .iter()
        .map(|server| control::Factory::create_control(server, &config.api.files.root))
        .collect();

    // get and convert the dependency tree
    let dependencies = config.dependencies.clone();
    let dependencies: dom::Dependencies = dependencies
        .0
        .iter()
        .map(|(device_id, deps)| {
            (
                dom::DeviceId::from(device_id),
                deps.iter().map(dom::DeviceId::from).collect(),
            )
        })
        .collect();

    // run the main code asynchronously
    info!("monitoring the network for activity...");
    let monitoring = {
        let sender = if provide_web_api {
            dom::communication::create_mpsc_sender(tx)
        } else {
            dom::communication::create_noop_sender()
        };
        let server_controls = server_controls.clone();
        let machines = machines.clone();
        let dependencies = dependencies.clone();
        rt.spawn(async move {
            let pinger = control::Factory::create_pinger(None);

            let mut monitor = monitor::Monitor::new(
                sender,
                ping_interval,
                server_controls,
                machines,
                dependencies,
                pinger,
            );

            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                interval.tick().await;
                monitor.run_once();
            }
        })
    };

    // create a list of all devices
    let mut devices: Vec<dom::Device> = servers
        .iter()
        .map(|server| dom::Device::Server(server.clone()))
        .collect();
    devices.extend(
        machines
            .iter()
            .map(|machine| dom::Device::Machine(machine.clone())),
    );

    // create a shared state of machines
    let shared_state: Arc<dom::communication::SharedStateMutex> =
        Arc::new(Mutex::new(dom::communication::SharedState::new(devices)));

    let sync = {
        let shared_state = shared_state.clone();
        rt.spawn(async move {
            if provide_web_api {
                let mut shared_state_sync = web::SharedStateSync::new(shared_state, rx);
                shared_state_sync.sync().await;
            } else {
                // make sure the task never ends
                loop {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        })
    };

    let rocket = rt.spawn(async move {
        if provide_web_api {
            // configure logging depending on cli arguments
            let mut log_level = rocket::config::LogLevel::Off;
            if args.verbose {
                log_level = rocket::config::LogLevel::Debug;
            } else if args.debug {
                log_level = rocket::config::LogLevel::Normal;
            }

            let ip = config.api.web.ip;
            let port = config.api.web.port;

            let server = web::Server::new(
                env::PKG_NAME,
                env::PKG_VERSION,
                config,
                shared_state,
                server_controls,
                dependencies,
                ip,
                port,
                log_level,
            );

            debug!("starting the web API...");
            if let Err(e) = server.launch().await {
                panic!("failed to launch Rocket-based web API: {}", e);
            }
        } else {
            // make sure the task never ends
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    });

    rt.block_on(async move {
        tokio::select! {
            _ = sigterm => exitcode::OK,
            _ = monitoring => exitcode::SOFTWARE,
            _ = sync => exitcode::SOFTWARE,
            _ = rocket => exitcode::SOFTWARE,
        }
    })
}

fn main() {
    // parse command line arguments
    let args: Opts = Opts::parse();

    let mut log_level = LevelFilter::Info;
    if args.verbose {
        log_level = LevelFilter::Trace;
    } else if args.debug {
        log_level = LevelFilter::Debug;
    }

    let _ = SimpleLogger::init(log_level, simplelog::Config::default());

    // read the configuration file
    info!("loading configuration from {}...", args.config);
    let config_result = configuration::parse_from_file(Path::new(&args.config));
    match &config_result {
        Err(e) => {
            error!("failed to load configuration from {}: {}", args.config, e);
            std::process::exit(exitcode::CONFIG);
        }
        _ => info!("configuration successfully loaded"),
    }

    let config = config_result.unwrap();

    // create the network
    let network_interface = match networking::get_network_interface(&config.network.interface) {
        Err(e) => {
            error!("{}", e);
            std::process::exit(exitcode::CONFIG);
        }
        Ok(r) => r,
    };

    if config.devices.is_empty() {
        error!("configuration doesn't contain any devices to monitor/control");
        std::process::exit(exitcode::CONFIG);
    }

    let configured_servers = configuration::get_servers(&config.devices);
    if configured_servers.is_empty() {
        error!("configuration doesn't contain any servers to control");
        std::process::exit(exitcode::CONFIG);
    }
    let configured_machines = configuration::get_machines(&config.devices);

    {
        // log the always on / off files
        let files = &config.api.files;
        info!("files API root directory: {}", files.root.to_str().unwrap());
    }

    // log the details of the configured network interface
    info!(
        "network: [{}] {}",
        network_interface.name,
        network_interface.mac.unwrap()
    );
    for ip in network_interface.ips.iter() {
        info!("  {}", ip);
    }

    {
        // log the ping configuration
        let ping = &config.network.ping;
        info!("ping: every {}s for {}s", ping.interval, ping.timeout);
    }

    // log the details of the configured servers
    info!("servers ({}):", configured_servers.len());
    for (_, server) in configured_servers.iter() {
        info!(
            "  {}@{}: {} [{}] ({}s)",
            server.ssh.username,
            server.machine.name,
            server.machine.ip,
            server.mac,
            server.machine.last_seen_timeout
        );
    }

    // log the details of the configured machines
    if !configured_machines.is_empty() {
        info!("machines ({}):", configured_machines.len());
        for (_, machine) in configured_machines.iter() {
            info!(
                "  {}: {} ({}s)",
                machine.name, machine.ip, machine.last_seen_timeout
            );
        }
    }

    info!("");

    // run the monitoring process
    let result = run(args, config, configured_servers, configured_machines);
    std::process::exit(result);
}
