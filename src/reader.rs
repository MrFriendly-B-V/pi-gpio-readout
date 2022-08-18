use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::{Duration, Instant};
use rppal::gpio::{InputPin, Level};

/// Read the GPIO pins and send their values to the printer
/// This functions starts a never ending loop, and thus does not return.
///
/// The provided `wait_time` determines how often the pin is read.
pub fn do_read(tx: Sender<(u8, Level)>, pins: &HashMap<u8, InputPin>, wait_time: Duration) -> ! {
    loop {
        for (idx, pin) in pins.iter() {
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