use std::str::FromStr;

use clap::Parser;
use simple_logger::SimpleLogger;

#[derive(Clone, Debug)]
enum Species {
    Name(String),
    Num(u32)
}

impl FromStr for Species {
    type Err = &'static str;    // The actual type doesn't matter since we never error, but it must implement `Display`
    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        Ok (s.parse::<u32>()
             .map (Species::Num)
             .unwrap_or_else (|_| Species::Name (s.to_string())))
    }
}

/// Shaoooh - Shiny Hunting Automaton On Original Hardware
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// Species to check
    species: Species,
    /// Reduce log verbosity
    #[arg(short, long, default_value_t = false)]
    quiet: bool,
}

fn main() {
    let args = Args::parse();
    let log_level = if args.quiet {
        log::Level::Info.to_level_filter()
    } else {
        log::Level::Trace.to_level_filter()
    };

    SimpleLogger::new()
        .with_level(log_level)
        .with_utc_timestamps()
        .init()
        .unwrap();


}
