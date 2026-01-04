use clap::Parser;
use simple_logger::SimpleLogger;

use crate::app::{Shaoooh, TransitionArg};

/// Shaoooh - Shiny Hunting Automaton On Original Hardware
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// Force the example config
    #[arg(short, long, default_value_t = false)]
    metamon: bool,
    /// Print the config and exit without running
    #[arg(short, long, default_value_t = false)]
    print: bool,
    /// Reduce log verbosity
    #[arg(short, long, default_value_t = false)]
    quiet: bool,
    /// Skip connection test
    #[arg(short, long, default_value_t = false)]
    skip_conn: bool,
}

pub fn main(cfg_fn: &dyn Fn() -> crate::app::Config, default_arg: TransitionArg) {
    let args = Args::parse();
    let log_level = if args.quiet {
        log::Level::Info.to_level_filter()
    } else {
        log::Level::Debug.to_level_filter()
    };

    SimpleLogger::new()
        .with_level(log_level)
        .with_utc_timestamps()
        .init()
        .unwrap();

    let config = if args.metamon {
        crate::app::Config::Ditto
    } else {
        cfg_fn()
    };

    if args.print {
        log::info!("Selected configuration: {}", config.info());
        log::info!("  {}", config.description());
    } else {
        log::info!("Starting Shaoooh Bot");

        let app = Shaoooh::new(config, default_arg);

        match app.serve(args.skip_conn) {
            Ok(_) => log::info!("Shaoooh done"),
            Err(e) => log::error!("{}", e),
        }

        log::info!("Shutdown");
    }
}
