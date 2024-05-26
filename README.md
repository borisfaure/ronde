![CI](https://github.com/borisfaure/ronde/actions/workflows/build_and_test.yml/badge.svg)
[![License](https://img.shields.io/badge/license-BSD--2--Clause-green.svg)](LICENSE.txt)

# Ronde

Ronde is a simple tool to monitor services and systems by running a defined
set of commands every X minutes. If one of them fails, a notification is sent
using the [PushOver API](https://pushover.net/).

A status page, with history, is generated on each run.
A sample status page is available at
[fau.re/ronde.sample](https://fau.re/ronde.sample/).

## Configuration

The configuration is done in a TOML file that is passed as argument to the
`ronde` binary.

A documented sample configuration file is [available in the repository as config.sample.toml](config.sample.toml).

## Building

To build the project, you need to have a working Rust environment. You can
install it using [rustup](https://rustup.rs/).

Once you have Rust installed, you can build the project using the following
command:

```sh
cargo build --release
```

The binary will be available in the `target/release` directory.

## Running

To run the project, you need to pass the path to the configuration file as an
argument:

```sh
ronde config.toml
```

The tool itself does not daemonize, so you need to use a tool like
[cronie](https://github.com/cronie-crond/cronie) or `systemd` to run it every
X minutes.

## Origin of the name

The name is used in French when guards are patrolling to ensure the safety of
a system.


## License

This project is available under the terms of the [BSD-2-Clause license](LICENSE.txt).

## Blog post

A blog post is available on [fau.re](https://fau.re/20240526_ronde/).
