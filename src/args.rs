use clap::{Parser, ValueEnum};
use tracing::Level;

#[derive(Debug, Parser)]
pub struct Args {
    /// Set the level of verbosity.
    /// When this argument is not given, the INFO level is used.
    /// When it is provided once, the DEBUG level is used.
    /// Otherwhise the TRACE level is used
    #[clap(short, parse(from_occurrences))]
    verbose: usize,
    /// The pin to read out.
    /// When this is unspecified, all of the Pi's pins will be read
    #[clap(long, short)]
    pub pin: Option<u8>,
    /// The input mode to use for all pins
    #[clap(short, long, value_enum)]
    pub input_mode: Option<InputMode>,
    /// Should the value be printed again if
    /// it's value is equal to the previous value from that pin.
    /// When unspecified, this defaults to true.
    #[clap(short, long)]
    pub reprint_if_previous_equal: Option<bool>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum InputMode {
    Regular,
    PullUp,
    PullDown,
}

impl Args {
    pub fn get_tracing_level(&self) -> Level {
        match self.verbose {
            0 => Level::INFO,
            1 => Level::DEBUG,
            _ => Level::TRACE,
        }
    }
}