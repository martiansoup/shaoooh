use clap::Parser;
use simple_logger::SimpleLogger;

use shaoooh::app::Shaoooh;

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
}

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
fn get_config() -> shaoooh::app::Config {
    let paths = shaoooh::app::CaptureControlPaths::new(
        "/dev/video0".to_string(),
        "/dev/ttyAMA0".to_string(),
    );
    shaoooh::app::Config::Shaoooh(paths)
}

#[cfg(all(target_os = "macos"))]
fn get_config() -> shaoooh::app::Config {
    use std::net::Ipv4Addr;

    shaoooh::app::Config::Bishaan(Ipv4Addr::new(192, 168, 68, 4))
}

#[cfg(not(any(all(target_arch = "aarch64", target_os = "linux"), target_os = "macos")))]
fn get_config() -> shaoooh::app::Config {
    shaoooh::app::Config::Ditto
}

#[tokio::main]
async fn main() {
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
        shaoooh::app::Config::Ditto
    } else {
        get_config()
    };

    if args.print {
        log::info!("Selected configuration: {}", config.info());
        log::info!("  {}", config.description());
    } else {
        log::info!("Starting Shaoooh Bot");

        let app = Shaoooh::new(config);

        match app.serve().await {
            Ok(_) => log::info!("Shaoooh done"),
            Err(e) => log::error!("{}", e),
        }

        log::info!("Shutdown");
    }
}
