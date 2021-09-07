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

fn run(
    args: Opts,
    config: configuration::Configuration,
    controllable_server: Box<dyn networking::controllable_server::ControllableServer>,
    pinger: Box<dyn networking::pinger::Pinger>,
    always_on: Box<dyn dom::always_on::AlwaysOn>,
) -> exitcode::ExitCode {
    let server = &config.server;

    // check if a manual option (wakeup / shutdown) has been provided
    if args.wakeup {
        info!("waking up {}...", server.machine.name);
        return match controllable_server.wakeup() {
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
        return match controllable_server.shutdown() {
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
        process(config, controllable_server, pinger, always_on)
    }
}

fn process(
    config: configuration::Configuration,
    controllable_server: Box<dyn networking::controllable_server::ControllableServer>,
    pinger: Box<dyn networking::pinger::Pinger>,
    always_on: Box<dyn dom::always_on::AlwaysOn>,
) -> exitcode::ExitCode {
    debug!("setting up signal handling for SIGTERM");
    let term = Arc::new(AtomicBool::new(false));
    if let Err(e) = signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term)) {
        error!("failed to setup signal handling: {}", e);
        return exitcode::SOFTWARE;
    }

    let mut monitor = monitor::Monitor::new(&config, controllable_server, pinger, always_on);

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

    // log the always on file
    let files = &config.files;
    info!("always on: {}", files.always_on);

    // log the details of the configured network interface
    info!(
        "network: [{}] {}",
        network_interface.name,
        network_interface.mac.unwrap()
    );
    for ip in network_interface.ips.iter() {
        info!("  {}", ip);
    }

    // log the ping configuration
    let ping = &config.network.ping;
    info!("ping: every {}s for {}s", ping.interval, ping.timeout);

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

    // log the details of the configured machines
    let machines = &config.machines;
    info!("machines ({}):", machines.len());
    for machine in machines.iter() {
        info!(
            "  {}: {} [{}] ({}s)",
            machine.name, machine.ip, machine.mac, machine.last_seen_timeout
        );
    }

    info!("");
    info!("monitoring the network for activity...");

    // instantiate an AlwaysOnFile
    let always_on = Box::new(dom::always_on_file::AlwaysOnFile::new(&config.files));

    // instantiate an Ssh2Server
    let controllable_server = Box::new(networking::ssh2_server::Ssh2Server::new(
        dom::machine::Server::new(&config.server),
    ));

    // instantiate the FastPinger
    let pinger = Box::new(networking::fast_pinger::FastPinger::new(None));

    // run the monitoring process
    std::process::exit(run(args, config, controllable_server, pinger, always_on));
}
