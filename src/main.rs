use clap::Clap;

use log::{error, info};
use simplelog::{LevelFilter, SimpleLogger};

use std::path::Path;

mod configuration;
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
        },
        _ => info!("configuration successfully loaded"),
    }

    let config = config_result.unwrap();

    // create the network
    let network = match networking::Networking::create(&config.network.interface) {
        Err(e) => {
            error!("{}", e);
            std::process::exit(exitcode::CONFIG);
        },
        Ok(r) => r,
    };

    // log the details of the configured network interface
    info!("network: [{}] {}", network.interface.name, network.interface.mac.unwrap());
    for ip in network.interface.ips.iter() {
        info!("  {}", ip);
    }

    // log the details of the configured server
    let server: configuration::machine::Server = config.server;
    info!("server: [{} ({})] {} / {}", server.machine.name, server.username, server.machine.mac, server.machine.ip);

    // check if a manual option (wakeup / shutdown) has been provided
    if args.wakeup {
        info!("waking up {}...", server.machine.name);
        match networking::wakeup(&server.machine) {
            Err(_)=> {
                error!("failed to wake up {}", server.machine.name);
                std::process::exit(exitcode::UNAVAILABLE);
            },
            Ok(_) => {
                info!("{} successfully woken up", server.machine.name);
                std::process::exit(exitcode::OK);
            },
        }
    } else if args.shutdown {
        info!("shutting down {}...", server.machine.name);
        match networking::shutdown::shutdown(&server) {
            Err(e)=> {
                error!("failed to shut down {}: {}", server.machine.name, e);
                std::process::exit(exitcode::UNAVAILABLE);
            },
            Ok(_) => {
                info!("{} successfully shut down", server.machine.name);
                std::process::exit(exitcode::OK);
            },
        }
    } else {
        unimplemented!()
    }
}
