use std::io::Write;
use std::sync::mpsc::Receiver;
use rppal::gpio::Level;
use crate::Args;
use std::thread;
use tracing::{debug, info};

pub fn start_printer(rx: Receiver<(u8, Level)>, args: &Args) {
    let reprint_if_eq = args.reprint_if_previous_equal;
    let print_only_bits = args.print_only_bits;

    thread::spawn(move || {
        let mut prev_values: [Option<Level>; 28] = [None; 28];

        // We count how many bits we've printed
        // This way we can flush stdio every byte
        // Otherwhise you get the output in massive chunks,
        // which is annoying
        let mut bit_count: u8 = 0;

        loop {
            match rx.recv() {
                Ok((pin_number, level)) => {
                    bit_count += 1;

                    // If the condition evaluates to true,
                    // we shouldn't print the value if it is equal to the previous value
                    if !reprint_if_eq && !print_only_bits {
                        // SAFETY: there are only 28 pins, so the idx
                        // will never exceed 28.
                        let prev_value = unsafe { prev_values.get_unchecked(pin_number as usize) };

                        // There is a previous value stored, check it for equality
                        // to the current level. We cant combine these if's yet.
                        // waiting on the if-let-else feature.
                        if let Some(prev_value) = prev_value {
                            if prev_value.eq(&level) {
                                continue;
                            }
                        }

                        prev_values[pin_number as usize] = Some(level);
                    }

                    if print_only_bits {
                        let numeric_value: u8 = match level {
                            Level::High => 1,
                            Level::Low => 0,
                        };

                        print!("{numeric_value}");
                    } else {
                        info!("Pin {}: {}", pin_number, level);
                    }

                    if bit_count == 8 {
                        let _ = std::io::stdout().flush();
                        bit_count = 0;
                    }
                },
                Err(e) => debug!("Recv Error: {e}"),
            }
        }
    });
}