use std::collections::HashMap;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use clap::Parser;
use rppal::gpio::{Gpio, InputPin, Level};
use tracing::{error, info, trace};
use crate::args::{Args, InputMode};
use anyhow::Result;

mod args;
mod printer;
mod reader;

fn main() {
    let args = Args::parse();
    configure_tracing(&args);

    if args.print_only_bits && args.pin.is_none() {
        error!("The flag `--print-only-bits` requires the `--pin` flag to given.");
        exit(1);
    }

    info!("Starting {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    if !nix::unistd::getuid().is_root() {
        error!("This program requires root access to run");
        exit(1);
    }

    defer_main(&args).expect("Error");
}

fn defer_main(args: &Args) -> Result<()> {
    info!("Configuring GPIO pins");

    let gpio = Gpio::new()?;
    let pins = configure_pins(&gpio, &args)?;

    // We print on a different thread to avoid the IO delays caused by printing to stdout
    let (tx, rx): (Sender<(u8, Level)>, Receiver<_>) = std::sync::mpsc::channel();

    trace!("Starting Printer");
    printer::start_printer(rx, &args);

    trace!("Calculating wait time");
    let wait_time = calculate_wait_time(&args);

    trace!("Starting read loop");
    reader::do_read(tx, &pins, wait_time);
}

/// Configure all pins the user wants to read.
/// Pins are put in the HashMap by their BCM pin number as the key.
fn configure_pins(gpio: &Gpio, args: &Args) -> Result<HashMap<u8, InputPin>> {
    let mut pins = HashMap::with_capacity(28);
    let input_mode = args.input_mode.as_ref().unwrap_or(&InputMode::Regular);

    if let Some(pin) = args.pin {
        pins.insert(pin, get_pin(&gpio, pin, input_mode)?);
    } else {
        // The RPI has 28 GPIO pins, starting at 0
        for i in 0..28 {
            pins.insert(i, get_pin(&gpio, i, input_mode)?);
        }
    }

    Ok(pins)
}

/// Get the provided `pin` as an input pin in the provided pin `mode`.
fn get_pin(gpio: &Gpio, pin: u8, mode: &InputMode) -> Result<InputPin> {
    trace!("Configuring pin {pin}");

    let pin = gpio.get(pin)?;
    Ok(match mode {
        InputMode::Regular => pin.into_input(),
        InputMode::PullDown => pin.into_input_pulldown(),
        InputMode::PullUp => pin.into_input_pullup(),
    })
}

/// Calculate the time a full read-loop duration should take
/// based on the user-provided bitrate.
fn calculate_wait_time(args: &Args) -> Duration {
    let bitrate = args.bitrate.unwrap_or(1000);
    let nanos = (10_000 / bitrate) * 100_000;

    info!("Reading every {nanos} nanoseconds ({bitrate} bits/s)");

    Duration::from_nanos(nanos)
}

/// Configure the tracing logger with the user-provided verbosity level.
fn configure_tracing(args: &Args) {
    let sub = tracing_subscriber::fmt()
        .compact()
        .with_max_level(args.get_tracing_level())
        .finish();
    tracing::subscriber::set_global_default(sub).expect("Configuring global tracing subscriber");
}
