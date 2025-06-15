use std::{process::Command, thread, time::Duration};

use simple_logger::SimpleLogger;
mod app;
mod control;
mod hunt;
mod vision;
use crate::app::Shaoooh;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::Level::Debug.to_level_filter())
        .with_utc_timestamps()
        .init()
        .unwrap();

    log::info!("Starting Shaooh");

    if std::fs::exists(Shaoooh::VIDEO_DEV).unwrap_or(false) {
        log::info!("Capture device at '{}' found", Shaoooh::VIDEO_DEV);
    } else {
        log::info!("Creating capture device");
        Command::new("sudo")
            .arg("modprobe")
            .arg("v4l2loopback")
            .arg(&format!("video_nr={}", Shaoooh::VIDEO_NUM))
            .arg("card_label=shaoooh")
            .status()
            .expect("Failed to create loopback device");
    }

    // TODO stream elsewhere as well, e.g.
    // ffmpeg -f v4l2 -i /dev/video0 -c:v libx264 -c:a copy -f tee -map 0:v "[f=v4l2]/dev/video250|[f=mpegts]udp://192.168.68.11:8090"
    // but only supports one connection and cannot reconnect
    Command::new("ffmpeg")
        .arg("-f")
        .arg("v4l2")
        .arg("-i")
        .arg("/dev/video0")
        .arg("-c:v")
        .arg("copy")
        .arg("-f")
        .arg("v4l2")
        .arg(Shaoooh::VIDEO_DEV)
        .arg("-loglevel")
        .arg("quiet")
        .spawn()
        .expect("Failed to start FFMPEG");

    thread::sleep(Duration::from_secs(2));

    // FFMPEG stream to virtual device

    // build our application with a single route
    let app = Shaoooh::new();

    match app.serve().await {
        Ok(_) => log::info!("Shaoooh done"),
        Err(e) => log::error!("{}", e),
    }

    log::info!("Shutdown");
}
