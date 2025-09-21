use std::net::Ipv4Addr;

use opencv::prelude::*;
use shaoooh::vision::{BishaanVision, BishaanVisionSocket};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::{broadcast, watch};

use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::Level::Debug.to_level_filter())
        .with_utc_timestamps()
        .init()
        .unwrap();

    log::info!("Starting Shaoooh Test : Bishaan Capture");

    let (t_frame_tx, mut t_frame_rx) = watch::channel(Mat::default());
    let (b_frame_tx, mut b_frame_rx) = watch::channel(Mat::default());
    let atomic = Arc::new(AtomicBool::new(true));
    let (error_tx_c, error_rx) = broadcast::channel(32);
    let error_tx = Arc::new(error_tx_c);

    let ip = Ipv4Addr::new(192, 168, 68, 4);

    tokio::spawn(async move {
        let vision = BishaanVisionSocket::new(ip, t_frame_tx, b_frame_tx, atomic, error_tx)
            .await
            .expect("Error creating vision thread");
        let vision_handle = tokio::spawn(vision.task());
    });

    //let vision = BishaanVision::new(t_frame_rx, b_frame_rx);

    let mut t_frame_id = 0;
    let mut b_frame_id = 0;

    loop {
        tokio::select! {
            _ = t_frame_rx.changed() => {
                let top_path = format!("3ds_frames/top_frame_{}.png", t_frame_id);
                let top = t_frame_rx.borrow().clone();
                opencv::imgcodecs::imwrite(&top_path, &top, &opencv::core::Vector::new());
                opencv::highgui::imshow("top", &top)
                    .unwrap_or_else(|_| panic!("Failed to show top window"));
                opencv::highgui::wait_key(1).expect("Event loop failed");
                t_frame_id += 1;
            }
            _ = b_frame_rx.changed() => {
                let bot_path = format!("3ds_frames/bot_frame_{}.png", b_frame_id);
                let bot = b_frame_rx.borrow().clone();
                opencv::imgcodecs::imwrite(&bot_path, &bot, &opencv::core::Vector::new());
                opencv::highgui::imshow("bot", &bot)
                    .unwrap_or_else(|_| panic!("Failed to show bottom window"));
                opencv::highgui::wait_key(1).expect("Event loop failed");
                b_frame_id += 1;
            }
        }
    }
}
