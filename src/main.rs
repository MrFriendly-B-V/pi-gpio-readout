use std::collections::HashMap;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};
use clap::Parser;
use rppal::gpio::{Gpio, InputPin, Level};
use tracing::{debug, error, info};
use crate::args::{Args, InputMode};
use anyhow::Result;

mod args;

fn main() {
    let args = Args::parse();
    configure_tracing(&args);

    info!("Starting {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    if !nix::unistd::getuid().is_root() {
        error!("This program requires root access to run");
        exit(1);
    }

    defer_main(&args).expect("Error");
}

fn defer_main(args: &Args) -> Result<()> {
    let input_mode = args.input_mode.as_ref().unwrap_or(&InputMode::Regular);

    info!("Configuring GPIO pins");

    let gpio = Gpio::new()?;
    let mut pins = HashMap::with_capacity(28);

    if let Some(pin) = args.pin {
        pins.insert(pin, get_pin(&gpio, pin, input_mode)?);
    } else {
        // The RPI has 28 GPIO pins, starting at 0
        for i in 0..28 {
            pins.insert(i, get_pin(&gpio, i, input_mode)?);
        }
    }

    info!("Starting readout");

    // We print on a different thread to avoid the IO delays caused by printing to stdout
    let (tx, rx): (Sender<(u8, Level)>, Receiver<_>) = std::sync::mpsc::channel();
    let reprint_if_eq = args.reprint_if_previous_equal.unwrap_or(true);

    thread::spawn(move || {
        let mut prev_values: [Option<Level>; 28] = [None; 28];

        loop {
            match rx.recv() {
                Ok(v) => {
                    // If the condition evaluates to true,
                    // we shouldn't print the value if it is equal to the previous value
                    if !reprint_if_eq {
                        // SAFETY: there are only 28 pins, so the idx
                        // will never exceed 28.
                        let prev_value = unsafe { prev_values.get_unchecked(v.0 as usize) };

                        // There is a previous value stored, check it for equality
                        // to the current level. We cant combine these if's yet.
                        // waiting on the if-let-else feature.
                        if let Some(prev_value) = prev_value {
                            if prev_value.eq(&v.1) {
                                continue;
                            }
                        }

                        prev_values[v.0 as usize] = Some(v.1);
                    }

                    info!("Pin {}: {}", v.0, v.1);
                },
                Err(e) => debug!("Recv Error: {e}"),
            }
        }
    });

    loop {
        let wait_time = Duration::from_nanos(500000);

        for (idx, pin) in pins.iter_mut() {
            let now = Instant::now();

            let level = pin.read();
            tx.send((*idx, level)).expect("RX channel closed");

            let runtime = now.elapsed();
            if let Some(remaining) = wait_time.checked_sub(runtime) {
                sleep(remaining);
            }
        }
    }
}

fn get_pin(gpio: &Gpio, pin: u8, mode: &InputMode) -> Result<InputPin> {
    let pin = gpio.get(pin)?;
    Ok(match mode {
        InputMode::Regular => pin.into_input(),
        InputMode::PullDown => pin.into_input_pulldown(),
        InputMode::PullUp => pin.into_input_pullup(),
    })
}



fn configure_tracing(args: &Args) {
    let sub = tracing_subscriber::fmt()
        .compact()
        .with_max_level(args.get_tracing_level())
        .finish();
    tracing::subscriber::set_global_default(sub).expect("Configuring global tracing subscriber");
}
