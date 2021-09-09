use clap::Clap;

use log::{debug, error, info};
use simplelog::{LevelFilter, SimpleLogger};

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

mod configuration;
mod dom;
mod monitor;
mod networking;

#[derive(Clap)]
#[clap(version = clap::crate_version!(), author = clap::crate_authors!())]
struct Opts {
    #[clap(short = 'c', long = "config", default_value = configuration::LOCATION)]
    config: String,

    #[clap(short = 'd', long = "debug", group = "verbosity")]
    debug: bool,
    #[clap(short = 'v', long = "verbose", group = "verbosity")]
    verbose: bool,

    #[clap(short = 's', long = "shutdown", group = "mode")]
    shutdown: bool,
    #[clap(short = 'w', long = "wakeup", group = "mode")]
    wakeup: bool,
}

#[allow(clippy::too_many_arguments)]
fn run(
    args: Opts,
    ping_interval: Duration,
    server: dom::machine::Server,
    machines: Vec<dom::machine::Machine>,
    wakeup_server: Box<dyn networking::wakeup_server::WakeupServer>,
    shutdown_server: Box<dyn networking::shutdown_server::ShutdownServer>,
    pinger: Box<dyn networking::pinger::Pinger>,
    always_on: Box<dyn dom::always_on::AlwaysOn>,
) -> exitcode::ExitCode {
    // check if a manual option (wakeup / shutdown) has been provided
    if args.wakeup {
        info!("waking up {}...", server.machine.name);
        return match wakeup_server.wakeup() {
            Err(_) => {
                error!("failed to wake up {}", server.machine.name);
                exitcode::UNAVAILABLE
            }
            Ok(_) => {
                info!("{} successfully woken up", server.machine.name);
                exitcode::OK
            }
        };
    } else if args.shutdown {
        info!("shutting down {}...", server.machine.name);
        return match shutdown_server.shutdown() {
            Err(e) => {
                error!("failed to shut down {}: {}", server.machine.name, e);
                exitcode::UNAVAILABLE
            }
            Ok(_) => {
                info!("{} successfully shut down", server.machine.name);
                exitcode::OK
            }
        };
    } else {
        process(
            ping_interval,
            server,
            machines,
            wakeup_server,
            shutdown_server,
            pinger,
            always_on,
        )
    }
}

fn process(
    ping_interval: Duration,
    server: dom::machine::Server,
    machines: Vec<dom::machine::Machine>,
    wakeup_server: Box<dyn networking::wakeup_server::WakeupServer>,
    shutdown_server: Box<dyn networking::shutdown_server::ShutdownServer>,
    pinger: Box<dyn networking::pinger::Pinger>,
    always_on: Box<dyn dom::always_on::AlwaysOn>,
) -> exitcode::ExitCode {
    debug!("setting up signal handling for SIGTERM");
    let term = Arc::new(AtomicBool::new(false));
    if let Err(e) = signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term)) {
        error!("failed to setup signal handling: {}", e);
        return exitcode::SOFTWARE;
    }

    let mut monitor = monitor::Monitor::new(
        ping_interval,
        server,
        machines,
        wakeup_server,
        shutdown_server,
        pinger,
        always_on,
    );

    while !term.load(Ordering::Relaxed) {
        monitor.run_once();

        std::thread::sleep(Duration::from_secs(1));
    }

    exitcode::OK
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

    if config.machines.is_empty() {
        error!("configuration doesn't contain any machines to monitor");
        std::process::exit(exitcode::CONFIG);
    }

    {
        // log the always on file
        let files = &config.files;
        info!("always on: {}", files.always_on);
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

    {
        // log the details of the configured server
        let server = &config.server;
        info!("server:");
        info!(
            "  {}@{}: {} [{}] ({}s)",
            server.username,
            server.machine.name,
            server.machine.ip,
            server.machine.mac,
            server.machine.last_seen_timeout
        );
    }

    {
        // log the details of the configured machines
        let machines = &config.machines;
        info!("machines ({}):", machines.len());
        for machine in machines.iter() {
            info!(
                "  {}: {} [{}] ({}s)",
                machine.name, machine.ip, machine.mac, machine.last_seen_timeout
            );
        }
    }

    info!("");
    info!("monitoring the network for activity...");

    let ping_interval = Duration::from_secs(config.network.ping.interval);

    // create the server DOM object from the parsed configuration
    let server = dom::machine::Server::from(&config.server);

    // create the machine DOM objects from the parsed configuration
    let mut machines = Vec::new();
    for machine in config.machines.iter() {
        machines.push(dom::machine::Machine::from(machine));
    }

    // instantiate a WakeOnLanServer
    let wakeup_server = Box::new(networking::wake_on_lan_server::WakeOnLanServer::new(
        &server,
    ));

    // instantiate a Ssh2ShutdownServer
    let shutdown_server = Box::new(networking::ssh2_shutdown_server::Ssh2ShutdownServer::new(
        &server,
    ));

    // instantiate the FastPinger
    let pinger = Box::new(networking::fast_pinger::FastPinger::new(None));

    // instantiate an AlwaysOnFile
    let always_on = Box::new(dom::always_on_file::AlwaysOnFile::new(&config.files));

    // run the monitoring process
    std::process::exit(run(
        args,
        ping_interval,
        server,
        machines,
        wakeup_server,
        shutdown_server,
        pinger,
        always_on,
    ));
}
