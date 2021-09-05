# home-monitor-rs

<p align="center">
    <a href="LICENSE"><img alt="License: GPL-2.0-later" src="https://img.shields.io/badge/license-GPLv2-blue"></a> <a href="https://github.com/Montellese/home-monitor-rs/pulls"><img alt="Pull Requests Welcome" src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg"></a> <a href="https://github.com/Montellese/home-monitor-rs/pulls"><img alt="Contributions Welcome" src="https://img.shields.io/badge/contributions-welcome-brightgreen.svg"></a>
    <br />
    <a href="https://app.travis-ci.com/github/Montellese/home-monitor-rs"><img alt="Travis CI Build Status" src="https://api.travis-ci.org/Montellese/home-monitor-rs.svg?branch=master"></a> <img alt="Rust Code Checks" src="https://github.com/Montellese/home-monitor-rs/actions/workflows/rust-code-checks.yml/badge.svg?branch=master">
</p>

`home-monitor-rs` is a re-write of [`home-monitor`](https://github.com/Montellese/home-monitor) in [Rust](https://www.rust-lang.org/).

`home-monitor-rs` is a service designed to run on an "always online" device (like a router or a Raspberry Pi) which constantly monitors a configurable list of devices (based on their IP addresses) to see if any of them are online. If all devices are offline a configured server is automatically turned off using SSH. If at least one of the configured devices is online the server is automatically turned on using Wake-on-LAN.

In addition to running `home-monitor-rs` as a service it can also be used to manually turn on or shut down the configured server.

- [home-monitor-rs](#home-monitor-rs)
  - [How to use](#how-to-use)
    - [Configuration](#configuration)
    - [Systemd Service](#systemd-service)
    - [Command Line Tool](#command-line-tool)
      - [Turn server on](#turn-server-on)
      - [Shut server down](#shut-server-down)
  - [How to develop](#how-to-develop)
    - [Requirements](#requirements)
    - [Build](#build)
    - [Run](#run)

## How to use

`home-monitor-rs` supports the following options / arguments:

```
USAGE:
    home-monitor-rs [FLAGS] [OPTIONS]

FLAGS:
    -d, --debug       
    -h, --help        Prints help information
    -s, --shutdown    
    -v, --verbose     
    -V, --version     Prints version information
    -w, --wakeup      

OPTIONS:
    -c, --config <config>    [default: /etc/home-monitor/home-monitor.json]
```

### Configuration

`home-monitor-rs` requires a JSON-based configuration file to be able to run properly. An example configuration is provided with `home-monitor.json.example`:

```json
{
    "files": {
        "alwaysOn": "/etc/home-monitor/alwayson"
    },
    "network": {
        "interface": "eth0",
        "ping": {
            "interval": 6,
            "timeout": 2
        }
    },
    "server": {
        "name": "My Server",
        "mac": "aa:bb:cc:dd:ee:ff",
        "ip": "192.168.1.255",
        "username": "foo",
        "password": "bar",
        "timeout": 60
    },
    "machines": [
        {
            "name": "My Machine",
            "mac": "ff:ee:dd:cc:bb:aa",
            "ip": "192.168.1.254",
            "timeout": 300
        }
    ]
}
```

The `machines` array can contain as many "machines" as necessary. Every configured machine will be monitored to determine the expected status of the configured `server`.

The `alwaysOn` file configuration option specifies a file which - if present - forces `home-monitor-rs` to turn the configured `server` on independent of the status of the configured `machines`.

### Systemd Service

To run `home-monitor-rs` as a systemd service use the provided `home-monitor-rs.service` systemd unit file. Once the unit file is in place use

```
sudo systemctl daemon-reload
```

to update systemd. Then use

```
sudo systemctl enable home-monitor-rs
```

to tell systemd that `home-monitor-rs` should automatically be started during boot.

You can control `home-monitor-rs` as a service using systemd's `systemctl` with the following commands

```
sudo systemctl [status|start|stop|restart|enable|disable] home-monitor-rs
```

### Command Line Tool

`home-monitor-rs` can also be used as a command line (CLI) tool to turn on or shut down the configured server.

#### Turn server on

To turn on the configured server use

```
home-monitor-rs --wakeup [-c <path to JSON configuration file>]
```

#### Shut server down

To shut the configured server down use

```
home-monitor-rs --shutdown [-c <path to JSON configuration file>]
```

## How to develop

### Requirements

To develop on `home-monitor-rs` you need a valid [Rust](https://www.rust-lang.org/) compiler installation. See [Install Rust](https://www.rust-lang.org/tools/install) for detailed instructions.

### Build

To build `home-monitor-rs` use Rust's package manager `cargo` by invoking

```
cargo build
```

### Run

To run the development version of `home-monitor-rs` use Rust's package manager `cargo` by invoking

```
cargo run -- -c <path to JSON configuration file>
```
