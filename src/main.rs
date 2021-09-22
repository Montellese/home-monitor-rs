use clap::Clap;

use log::{debug, error, info};
use simplelog::{LevelFilter, SimpleLogger};

use std::path::Path;
use std::time::Duration;

mod configuration;
mod dom;
mod monitor;
mod networking;
mod utils;
mod web;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    config: configuration::Configuration,
    ping_interval: Duration,
    server: dom::Server,
    machines: Vec<dom::Machine>,
    wakeup_server: Box<dyn networking::WakeupServer>,
    shutdown_server: Box<dyn networking::ShutdownServer>,
    pinger: Box<dyn networking::Pinger>,
    always_off: Box<dyn utils::AlwaysOff>,
    always_on: Box<dyn utils::AlwaysOn>,
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
            args,
            config,
            ping_interval,
            server,
            machines,
            wakeup_server,
            shutdown_server,
            pinger,
            always_off,
            always_on,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn process(
    args: Opts,
    config: configuration::Configuration,
    ping_interval: Duration,
    server: dom::Server,
    machines: Vec<dom::Machine>,
    wakeup_server: Box<dyn networking::WakeupServer>,
    shutdown_server: Box<dyn networking::ShutdownServer>,
    pinger: Box<dyn networking::Pinger>,
    always_off: Box<dyn utils::AlwaysOff>,
    always_on: Box<dyn utils::AlwaysOn>,
) -> exitcode::ExitCode {
    // create the tokio runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(web::Server::get_num_workers())
        .thread_name(web::Server::get_thread_name(PKG_NAME))
        .enable_all()
        .build()
        .expect("failed to build a tokio runtime");

    // setup SIGINT signal handling
    debug!("setting up signal handling for SIGTERM");
    let sigterm = tokio::signal::ctrl_c();

    // run the main code asynchronously
    info!("monitoring the network for activity...");
    let monitoring = rt.spawn(async move {
        let mut monitor = monitor::Monitor::new(
            ping_interval,
            server,
            machines,
            wakeup_server,
            shutdown_server,
            pinger,
            always_off,
            always_on,
        );

        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            monitor.run_once();
        }
    });

    let rocket = rt.spawn(async move {
        // only start the web API if a valid port is configured
        if config.api.web.port > 0 {
            // configure logging depending on cli arguments
            let mut log_level = rocket::config::LogLevel::Off;
            if args.verbose {
                log_level = rocket::config::LogLevel::Debug;
            } else if args.debug {
                log_level = rocket::config::LogLevel::Normal;
            }

            let server = web::Server::new(PKG_NAME, PKG_VERSION);

            debug!("starting the web API...");
            if let Err(e) = server
                .launch(config.api.web.ip, config.api.web.port, log_level)
                .await
            {
                panic!("failed to launch Rocket-based web API: {}", e);
            }
        }
    });

    rt.block_on(async move {
        tokio::select! {
            _ = sigterm => exitcode::OK,
            _ = monitoring => exitcode::SOFTWARE,
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

    if config.machines.is_empty() {
        error!("configuration doesn't contain any machines to monitor");
        std::process::exit(exitcode::CONFIG);
    }

    {
        // log the always on / off files
        let files = &config.api.files;
        info!("always on: {}", files.always_on);
        info!("always off: {}", files.always_off);
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

    let ping_interval = Duration::from_secs(config.network.ping.interval);

    // create the server DOM object from the parsed configuration
    let server = dom::Server::from(&config.server);

    // create the machine DOM objects from the parsed configuration
    let mut machines = Vec::new();
    for machine in config.machines.iter() {
        machines.push(dom::Machine::from(machine));
    }

    // instantiate a WakeOnLanServer
    let wakeup_server = Box::new(networking::WakeOnLanServer::new(&server));

    // instantiate a Ssh2ShutdownServer
    let shutdown_server = Box::new(networking::Ssh2ShutdownServer::new(&server));

    // instantiate the FastPinger
    let pinger = Box::new(networking::FastPinger::new(None));

    // instantiate an AlwaysOffFile / AlwaysOnFile
    let always_off = Box::new(utils::AlwaysOffFile::from(&config.api.files));
    let always_on = Box::new(utils::AlwaysOnFile::from(&config.api.files));

    // run the monitoring process
    let result = run(
        args,
        config,
        ping_interval,
        server,
        machines,
        wakeup_server,
        shutdown_server,
        pinger,
        always_off,
        always_on,
    );
    std::process::exit(result);
}
