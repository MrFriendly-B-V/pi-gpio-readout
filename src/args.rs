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
    /// When `--print-only-bits` is specified, this option is ignored
    #[clap(long, action)]
    pub reprint_if_previous_equal: bool,
    /// Print only the bits, rather than HIGH/LOW and a timestamp
    /// This only works is `--pin` is provided.
    /// Enabling this option, ignores whatever is set for `--reprint-if-previous-equal`
    #[clap(long, action)]
    pub print_only_bits: bool,
    /// How many bits per seconds should be read.
    /// Default: 1000
    #[clap(long)]
    pub bitrate: Option<u64>,
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