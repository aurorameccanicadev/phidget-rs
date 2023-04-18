// phidget-rs/examples/temperature.rs
//
// Copyright (c) 2023, Frank Pagliughi
//
// This file is an example application for the 'phidget-rs' library.
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! Rust Phidget example application to read temperature.
//!

use clap::{arg, value_parser, ArgAction};
use phidget::Phidget;
use std::{thread, time::Duration};

// Open/connect timeout
const TIMEOUT: Duration = Duration::from_millis(5000);

// The package version is used as the app version
const VERSION: &str = env!("CARGO_PKG_VERSION");

// --------------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    let opts = clap::Command::new("temperature")
        .version(VERSION)
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Phidget Temperature Monitoring Example")
        .disable_help_flag(true)
        .arg(
            arg!(--help "Print help information")
                .short('?')
                .action(ArgAction::Help),
        )
        .arg(
            arg!(-s --serial <serial_num> "Specify the serial number of the device to open")
                .value_parser(value_parser!(i32)),
        )
        .arg(
            arg!(-c --channel <chan> "Specify the channel number of the device to open")
                .value_parser(value_parser!(i32)),
        )
        .arg(
            arg!(-p --port <port> "Use a specific port on a VINT hub directly")
                .value_parser(value_parser!(i32)),
        )
        .arg(arg!(-h --hub "Use a hub VINT input port directly"))
        .get_matches();

    let use_hub = opts.get_flag("hub");

    println!("Opening Phidget temperature sensor...");
    let mut temp = phidget::TemperatureSensor::new();

    // Whether we should use a hub port directly as the input,
    // and if so, which one?
    temp.set_is_hub_port_device(use_hub)?;
    if let Some(&port) = opts.get_one::<i32>("port") {
        temp.set_hub_port(port)?;
    }

    // Some other device selection filters...
    if let Some(&num) = opts.get_one::<i32>("serial") {
        temp.set_serial_number(num)?;
    }

    if let Some(&chan) = opts.get_one::<i32>("channel") {
        temp.set_channel(chan)?;
    }

    temp.open_wait(TIMEOUT)?;

    if use_hub {
        let port = temp.hub_port()?;
        println!("Opened on hub port: {}", port);
    }

    let t = temp.temperature()?;
    println!("Temperature: {}", t);

    temp.set_on_temperature_change_handler(|_, t: f64| {
        println!("Temperature: {}", t);
    })?;

    // ^C handler wakes up the main thread
    ctrlc::set_handler({
        let thr = thread::current();
        move || {
            println!("\nExiting...");
            thr.unpark();
        }
    })
    .expect("Error setting Ctrl-C handler");

    // Block until a ^C wakes us up to exit.
    thread::park();
    Ok(())
}
