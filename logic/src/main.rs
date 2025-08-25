use simple_logger::SimpleLogger;

use shaoooh::app::Shaoooh;

#[cfg(target_arch = "aarch64")]
fn get_config() -> shaoooh::app::Config {
    let paths = shaoooh::app::CaptureControlPaths::new(
        "/dev/video0".to_string(),
        "/dev/ttyAMA0".to_string(),
    );
    shaoooh::app::Config::Shaoooh(paths)
}

#[cfg(not(target_arch = "aarch64"))]
fn get_config() -> shaoooh::app::Config {
    shaoooh::app::Config::Ditto
}

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::Level::Debug.to_level_filter())
        .with_utc_timestamps()
        .init()
        .unwrap();

    log::info!("Starting Shaoooh Bot");

    let app = Shaoooh::new(get_config());

    match app.serve().await {
        Ok(_) => log::info!("Shaoooh done"),
        Err(e) => log::error!("{}", e),
    }

    log::info!("Shutdown");
}
