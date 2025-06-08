use simple_logger::SimpleLogger;
mod app;
mod control;
mod hunt;
use crate::app::Shaoooh;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::Level::Debug.to_level_filter())
        .with_utc_timestamps()
        .init()
        .unwrap();

    log::info!("Starting Shaooh");

    // build our application with a single route
    let app = Shaoooh::new();

    match app.serve().await {
        Ok(_) => log::info!("Shaoooh done"),
        Err(e) => log::error!("{}", e),
    }

    log::info!("Shutdown");
}
