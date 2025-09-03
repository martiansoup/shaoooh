use std::{net::Ipv4Addr, path::Path, time::Duration};

use opencv::{
    core::{BORDER_DEFAULT, CV_8S, CV_32S, Point, Vector},
    highgui::{WINDOW_GUI_EXPANDED, WINDOW_GUI_NORMAL, WINDOW_KEEPRATIO, WINDOW_NORMAL},
    imgcodecs::{IMREAD_COLOR, IMREAD_GRAYSCALE},
    imgproc::{THRESH_BINARY, TM_CCORR_NORMED},
    prelude::*,
};
use shaoooh::vision::{BishaanVision, compat};
use tokio::sync::watch;

use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::Level::Debug.to_level_filter())
        .with_utc_timestamps()
        .init()
        .unwrap();

    log::info!("Starting Shaoooh Test : Bishaan Process");

    let (t_frame_tx, mut t_frame_rx) = watch::channel(Mat::default());
    let (b_frame_tx, mut b_frame_rx) = watch::channel(Mat::default());

    let ip = Ipv4Addr::new(192, 168, 68, 4);

    tokio::spawn(async move {
        let bot = opencv::imgcodecs::imread("3ds_frames/bot_frame_0.png", IMREAD_COLOR).unwrap();
        b_frame_tx.send(bot);
        let mut im_index = 0;
        let mut im_path = format!("3ds_frames/top_frame_{}.png", im_index);
        let mut path = Path::new(&im_path);
        while path.exists() {
            let top = opencv::imgcodecs::imread(&im_path, IMREAD_COLOR).expect("Failed to read");
            t_frame_tx.send(top);

            im_index += 1;
            im_path = format!("3ds_frames/top_frame_{}.png", im_index);
            path = Path::new(&im_path);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    //let vision = BishaanVision::new(t_frame_rx, b_frame_rx);
    opencv::highgui::named_window("th", WINDOW_KEEPRATIO | WINDOW_GUI_EXPANDED);
    opencv::highgui::named_window("top_hp", WINDOW_KEEPRATIO | WINDOW_GUI_EXPANDED);

    let ref_img = opencv::imgcodecs::imread("static/usum_shiny_star.png", IMREAD_GRAYSCALE)
        .expect("Couldn't read image");

    let mut mask = Mat::default();

    opencv::imgproc::threshold(&ref_img, &mut mask, 50.0, 255.0, THRESH_BINARY)
        .expect("Failed to create mask");

    loop {
        let top = t_frame_rx.borrow().clone();
        let bottom = b_frame_rx.borrow().clone();

        opencv::highgui::imshow("top", &top);

        // Conert to grey first?
        let mut grey = Mat::default();
        compat::cvt_color(&top, &mut grey, opencv::imgproc::COLOR_BGR2GRAY, 0);

        let mut hsv = Mat::default();
        compat::cvt_color(&top, &mut hsv, opencv::imgproc::COLOR_BGR2HSV, 0)
            .expect("Failed to convert colour");
        let mut thresholded_ylw = Mat::default();
        let lower = Vector::from_slice(&[25.0, 32.0, 100.0]);
        let upper = Vector::from_slice(&[40.0, 200.0, 255.0]);
        opencv::core::in_range(&hsv, &lower, &upper, &mut thresholded_ylw);
        opencv::highgui::imshow("yellow", &thresholded_ylw);

        let mut thresholded = Mat::default();
        opencv::imgproc::threshold(&grey, &mut thresholded, 220.0, 255.0, THRESH_BINARY)
            .expect("Failed to apply threshold");
        opencv::highgui::imshow("th", &thresholded);

        let mut hp = Mat::default();
        opencv::imgproc::sobel(&thresholded, &mut hp, -1, 1, 1, 5, 1.0, 0.0, BORDER_DEFAULT);

        let mut result = Mat::default();
        opencv::imgproc::match_template(
            &thresholded_ylw,
            &ref_img,
            &mut result,
            TM_CCORR_NORMED,
            &Mat::default(),
        )
        .expect("Failed to convolve");

        let mut max_val = 0.0;
        let mut max_loc = Point::default();

        opencv::core::min_max_loc(
            &result,
            None,
            Some(&mut max_val),
            None,
            Some(&mut max_loc),
            &opencv::core::no_array(),
        )
        .expect("min max failed");

        //println!("Correlation = {}", max_val);
        if max_val > 0.8 && !(max_val < 0.8) && (max_val < 2.0) {
            println!("MET");
        }

        opencv::highgui::imshow("top_hp", &hp);

        opencv::highgui::imshow("bottom", &bottom);
        opencv::highgui::wait_key(1).expect("Event loop failed");

        std::thread::sleep(Duration::from_millis(50));
    }
}
