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
    - [Web / REST API](#web--rest-api)
      - [GET /config](#get-config)
      - [GET /status](#get-status)
      - [GET /always_off](#get-always_off)
      - [POST /always_off](#post-always_off)
      - [DELETE /always_off](#delete-always_off)
      - [GET /always_on](#get-always_on)
      - [POST /always_on](#post-always_on)
      - [DELETE /always_on](#delete-always_on)
      - [PUT /wakeup](#put-wakeup)
      - [PUT /shutdown](#put-shutdown)
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
    "network": {
        "interface": "eth0",
        "ping": {
            "interval": 6,
            "timeout": 2
        }
    },
    "api": {
        "files": {
            "alwaysOff": "/etc/home-monitor/alwaysoff",
            "alwaysOn": "/etc/home-monitor/alwayson"
        },
        "web": {
            "ip": "127.0.0.1",
            "port": 8000
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

The `alwaysOff` file configuration option specifies a file which - if present - forces `home-monitor-rs` to turn the configured `server` off independent of the status of the configured `machines`. Similarly the `alwaysOn` file configuration option specifies a file which - if present - forces `home-monitor-rs` to turn the configured `server` on independent of the status of the configured `machines`.

The `web` configuration in the `api` section can be used to configure an optional web / REST API. If the `web` section is completely missing of the `port` option is `0` the web / REST API is not started. If `ip` contains a valid IP address and `port` a valid HTTP port the web / REST API is automatically started.

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

### Web / REST API

`home-monitor-rs` provides an optional web / REST API to observe and control its behaviour. Based on the configured IP address and port the REST API is available under `http://<IP>:<PORT>/api/v1/` followed by a specific REST endpoint. The following chapters describe the available endpoints.

#### GET /config

This REST endpoints returns the currently used / loaded configuration in JSON format.

#### GET /status

This REST endpoint returns the current status of the monitored server and machines in JSON format.

#### GET /always_off

This REST endpoints returns the current status of the `alwaysOff` feature in the following JSON format:
```json
{ "always_off": true }
```

#### POST /always_off

This REST endpoint activates the `alwaysOff` feature (independent of whether it was already active or not) and returns the new status in the JSON format described in [GET /always_off](#get-always_off).

#### DELETE /always_off

This REST endpoint deactivates the `alwaysOff` feature (independent of whether it was already inactive or not) and returns the new status in the JSON format described in [GET /always_off](#get-always_off).

#### GET /always_on

This REST endpoints returns the current status of the `alwaysOn` feature in the following JSON format:
```json
{ "always_on": true }
```

#### POST /always_on

This REST endpoint activates the `alwaysOn` feature (independent of whether it was already active or not) and returns the new status in the JSON format described in [GET /always_on](#get-always_on).

#### DELETE /always_on

This REST endpoint deactivates the `alwaysOn` feature (independent of whether it was already inactive or not) and returns the new status in the JSON format described in [GET /always_on](#get-always_on).

#### PUT /wakeup

This REST endpoint forces `home-monitor-rs` to wake up the configured server independent of its current status or the status of the monitored machines. This is the same functionality as provided by the [Command Line Tool](#command-line-tool).

#### PUT /shutdown

This REST endpoint forces `home-monitor-rs` to shut down the configured server independent of its current status or the status of the monitored machines. This is the same functionality as provided by the [Command Line Tool](#command-line-tool).

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
