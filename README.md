# home-monitor-rs

<p align="center">
    <a href="LICENSE"><img alt="License: GPL-2.0-later" src="https://img.shields.io/badge/license-GPLv2-blue"></a> <a href="https://github.com/Montellese/home-monitor-rs/pulls"><img alt="Pull Requests Welcome" src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg"></a> <a href="https://github.com/Montellese/home-monitor-rs/pulls"><img alt="Contributions Welcome" src="https://img.shields.io/badge/contributions-welcome-brightgreen.svg"></a>
    <br />
    <a href="https://app.travis-ci.com/github/Montellese/home-monitor-rs"><img alt="Travis CI Build Status" src="https://api.travis-ci.org/Montellese/home-monitor-rs.svg?branch=master"></a> <img alt="Rust Code Checks" src="https://github.com/Montellese/home-monitor-rs/actions/workflows/rust-code-checks.yml/badge.svg?branch=master">
</p>

`home-monitor-rs` is a re-write of [`home-monitor`](https://github.com/Montellese/home-monitor) in [Rust](https://www.rust-lang.org/) with additional features like a web / REST API.

`home-monitor-rs` is a service designed to run on an "always online" device (like a router or a Raspberry Pi) which constantly monitors a configurable list of devices (based on their IP addresses) to see if any of them are online. Depending on the configured dependencies between servers and machines a server is automatically turned off using SSH if all relevant machines are offline. It at least one of the machines is online the server is automatically turned on using Wake-on-LAN.

In addition to running `home-monitor-rs` as a service it can also be used to manually turn on or shut down the configured servers.

- [home-monitor-rs](#home-monitor-rs)
  - [How to use](#how-to-use)
    - [Configuration](#configuration)
    - [Systemd Service](#systemd-service)
    - [Web / REST API](#web--rest-api)
      - [GET /config](#get-config)
      - [GET /status](#get-status)
      - [GET /server/\<server\>/status](#get-serverserverstatus)
      - [GET /server/\<server\>/always_off](#get-serverserveralways_off)
      - [POST /server/\<server\>/always_off](#post-serverserveralways_off)
      - [DELETE /server/\<server\>/always_off](#delete-serverserveralways_off)
      - [GET /server/\<server\>/always_on](#get-serverserveralways_on)
      - [POST /server/\<server\>/always_on](#post-serverserveralways_on)
      - [DELETE /server/\<server\>/always_on](#delete-serverserveralways_on)
      - [PUT /server/\<server\>/wakeup](#put-serverserverwakeup)
      - [PUT /server/\<server\>/shutdown](#put-serverservershutdown)
    - [Command Line Tool](#command-line-tool)
      - [Turn server(s) on](#turn-servers-on)
      - [Shut server(s) down](#shut-servers-down)
  - [How to develop](#how-to-develop)
    - [Requirements](#requirements)
    - [Build](#build)
    - [Run](#run)
    - [Debian Packaging](#debian-packaging)

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
    -c, --config <FILE>           Path to the JSON configuration file [default: /etc/home-monitor/home-monitor.json]
    -s, --shutdown <SERVER>...    Shut down the specified server(s)
    -w, --wakeup <SERVER>...      Wake up the specified server(s)
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
            "root": "/etc/home-monitor-rs/"
        },
        "web": {
            "ip": "127.0.0.1",
            "port": 8000
        }
    },
    "devices": {
        "myserver": {
            "name": "My Server",
            "mac": "aa:bb:cc:dd:ee:ff",
            "ip": "192.168.1.255",
            "username": "foo",
            "password": "bar",
            "timeout": 60
        },
        "mymachine": {
            "name": "My Machine",
            "ip": "192.168.1.254",
            "timeout": 300
        }
    },
    "dependencies": {
        "myserver": {
            "mymachine"
        }
    }
}
```

The `devices` object can contain as many "devices" as necessary and is a combination of "servers" and "machines". Every configured device will be monitored to determine the expected status of the server depending on the device to be online (through the `dependencies` object). A server can also depend on one or more other servers.

Any device which should be controlled by `home-monitor-rs` must be configured with a `mac`, `username` and `password` whereas machines which are just monitored don't need these properties.

The `files.root` configuration option in the `api` section specifies the root directory for the file based API. `home-monitor-rs` automatically creates a new sub-directory in the `root` directory for every server to be controlled. Within that subdirectory two files can be created:
* if the `alwaysoff` file is present it forces `home-monitor-rs` to shut the configured server down independent of the status of the machines.
* if the `alwayson` file is present it forces `home-monitor-rs` to turn the configured server on independent of the status of the configured machines.

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

A detailed and automatically generated [OpenAPI specification](https://www.openapis.org/) is available through [Swagger UI](https://swagger.io/tools/swagger-ui/) and [RapiDoc](https://mrin9.github.io/RapiDoc/) under `http://<IP>:<PORT>/docs/swagger` and `http://<IP>:<PORT>/docs/rapidoc`.

#### GET /config

This REST endpoint returns the currently used / loaded configuration in JSON format.

#### GET /status

This REST endpoint returns the current status of the configured devices in JSON format.

#### GET /server/\<server\>/status

This REST endpoint returns the current status of the given server and the machines it depends on in JSON format.

#### GET /server/\<server\>/always_off

This REST endpoint returns the current status of the `alwaysoff` feature for the given server in the following JSON format:
```json
{ "always_off": true }
```

#### POST /server/\<server\>/always_off

This REST endpoint activates the `alwaysoff` feature (independent of whether it was already active or not) for the given server and returns the new status in the JSON format described in [GET /server/\<server\>/always_off](#get-serverserveralways_off).

#### DELETE /server/\<server\>/always_off

This REST endpoint deactivates the `alwaysoff` feature (independent of whether it was already inactive or not) for the given server and returns the new status in the JSON format described in [GET /server/\<server\>/always_off](#get-serverserveralways_off).

#### GET /server/\<server\>/always_on

This REST endpoint returns the current status of the `alwayson` feature for the given server in the following JSON format:
```json
{ "always_on": true }
```

#### POST /server/\<server\>/always_on

This REST endpoint activates the `alwayson` feature (independent of whether it was already active or not) for the given server and returns the new status in the JSON format described in [GET /server/\<server\>/always_on](#get-serverserveralways_on).

#### DELETE /server/\<server\>/always_on

This REST endpoint deactivates the `alwayson` feature (independent of whether it was already inactive or not) for the given server and returns the new status in the JSON format described in [GET /server/\<server\>/always_on](#get-serverserveralways_on).

#### PUT /server/\<server\>/wakeup

This REST endpoint forces `home-monitor-rs` to wake up the given server independent of its current status or the status of the monitored machines. This is the same functionality as provided by the [Command Line Tool](#command-line-tool).

#### PUT /server/\<server\>/shutdown

This REST endpoint forces `home-monitor-rs` to shut down the given server independent of its current status or the status of the monitored machines. This is the same functionality as provided by the [Command Line Tool](#command-line-tool).

### Command Line Tool

`home-monitor-rs` can also be used as a command line (CLI) tool to turn on or shut down the configured server.

#### Turn server(s) on

To turn on one or more configured servers use

```
home-monitor-rs --wakeup myserver [-c <path to JSON configuration file>]
```

#### Shut server(s) down

To shut one or more configured servers down use

```
home-monitor-rs --shutdown myserver [-c <path to JSON configuration file>]
```

## How to develop

### Requirements

To develop on `home-monitor-rs` you need a valid [Rust](https://www.rust-lang.org/) compiler installation. See [Install Rust](https://www.rust-lang.org/tools/install) for detailed instructions.

#### Ubuntu

To build all dependencies of `home-monitor-rs` the following system packages need to be installed:

```
sudo apt-get install pkg-config libssl-dev
```

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

*NOTE*: If your user does not have the permissions to create RAW sockets you first need to set the necessary capability using
```
sudo setcap cap_net_raw=eip target/debug/home-monitor-rs
```

### Debian Packaging
`home-monitor-rs` provides the necessary configuration to build a Debian package (including a `systemd` service file) using [`cargo-deb`](https://github.com/mmstick/cargo-deb).

First install `cargo-deb` using
```
cargo install cargo-deb
```
and then run
```
cargo deb
```
to build a Debian package placed under `target/debian/`.

*NOTE*: The `systemd` service file is installed and enabled when installing the Debian package but `home-monitor-rs.service` is not automatically started because the user first has to provide a proper configuration at `/etc/home-monitor-rs/home-monitor-rs.json` based on the example configuration located at `/etc/home-monitor-rs/home-monitor-rs.json.example`.
