use std::time::{Duration, SystemTime};

use crate::app::ShaooohError;
use crate::vision::{BotVision, ProcessingResult, compat};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use opencv::{
    core::{Point, Vector},
    imgcodecs::IMREAD_GRAYSCALE,
    imgproc::{THRESH_BINARY, TM_CCORR_NORMED},
    prelude::*,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UdpSocket, tcp::OwnedWriteHalf},
    sync::{broadcast, watch},
};

use super::{NTRPacket, Processing};

enum Frame {
    None,
    Top(Mat),
    Bottom(Mat),
}

pub struct BishaanVision {
    rx_top: watch::Receiver<Mat>,
    rx_bottom: watch::Receiver<Mat>,
    encoded_top: Vector<u8>,
    encoded_bottom: Vector<u8>,
    ref_shiny_star: Mat,
    shiny_star_mask: Mat,
}

pub struct BishaanVisionSocket {
    tx_top: watch::Sender<Mat>,
    tx_bottom: watch::Sender<Mat>,
    tcp_sock: Option<TcpStream>,
    img_socket: UdpSocket,
    top_frame_num: u8,
    top_frame_seq: u8,
    top_screen_buf: Vec<u8>,
    bot_frame_num: u8,
    bot_frame_seq: u8,
    bot_screen_buf: Vec<u8>,
    can_heartbeat: Arc<AtomicBool>,
    last_fps: SystemTime,
    last_frame_count: usize,
    error_tx: Arc<broadcast::Sender<ShaooohError>>,
}

impl BotVision for BishaanVision {
    fn process_next_frame(
        &mut self,
        processing: &[super::Processing],
    ) -> Option<Vec<ProcessingResult>> {
        {
            let bottom = { self.rx_bottom.borrow().clone() };
            let top = { self.rx_top.borrow().clone() };
            if !bottom.empty() {
                opencv::imgcodecs::imencode(
                    ".png",
                    &bottom,
                    &mut self.encoded_bottom,
                    &Vector::new(),
                )
                .expect("Failed to encode frame");
                opencv::highgui::imshow("bottom", &bottom)
                    .unwrap_or_else(|_| panic!("Failed to show bottom window"));
                opencv::highgui::wait_key(1).expect("Event loop failed");
            }
            if !top.empty() {
                opencv::imgcodecs::imencode(".png", &top, &mut self.encoded_top, &Vector::new())
                    .expect("Failed to encode frame");
                opencv::highgui::imshow("top", &top)
                    .unwrap_or_else(|_| panic!("Failed to show top window"));
                opencv::highgui::wait_key(1).expect("Event loop failed");
            }
            Some(
                processing
                    .iter()
                    .map(|p| self.process(p, &top, &bottom))
                    .collect(),
            )
        }
    }

    fn read_frame(&self) -> &[u8] {
        self.encoded_top.as_slice()
    }

    fn read_frame2(&self) -> &[u8] {
        self.encoded_bottom.as_slice()
    }
}

impl BishaanVision {
    pub fn new(rx_top: watch::Receiver<Mat>, rx_bottom: watch::Receiver<Mat>) -> Self {
        let ref_shiny_star =
            opencv::imgcodecs::imread("static/usum_shiny_star.png", IMREAD_GRAYSCALE)
                .expect("Couldn't read image");

        let mut shiny_star_mask = Mat::default();

        opencv::imgproc::threshold(
            &ref_shiny_star,
            &mut shiny_star_mask,
            50.0,
            255.0,
            THRESH_BINARY,
        )
        .expect("Failed to create mask");

        BishaanVision {
            rx_top,
            rx_bottom,
            encoded_top: Vector::default(),
            encoded_bottom: Vector::default(),
            ref_shiny_star,
            shiny_star_mask,
        }
    }

    fn shiny_star(&self, frame: &Mat, target: u32) -> ProcessingResult {
        let mut hsv = Mat::default();
        compat::cvt_color(&frame, &mut hsv, opencv::imgproc::COLOR_BGR2HSV, 0)
            .expect("Failed to convert colour");
        let mut thresholded_ylw = Mat::default();
        let lower = Vector::from_slice(&[25.0, 32.0, 100.0]);
        let upper = Vector::from_slice(&[40.0, 200.0, 255.0]);
        opencv::core::in_range(&hsv, &lower, &upper, &mut thresholded_ylw);

        let mut result = Mat::default();
        opencv::imgproc::match_template(
            &thresholded_ylw,
            &self.ref_shiny_star,
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

        let met = max_val > 0.5 && !(max_val < 0.5) && (max_val < 2.0);
        log::debug!("Value = {} (met={})", max_val, met);

        ProcessingResult {
            process: Processing::USUMShinyStar(target),
            met,
            species: target,
            shiny: met,
        }
    }

    fn bottom(&self, frame: &Mat, threshold: f64, inv: bool) -> ProcessingResult {
        let mut bot_grey = Mat::default();
        compat::cvt_color(&frame, &mut bot_grey, opencv::imgproc::COLOR_BGR2GRAY, 0)
            .expect("Failed to convert colour");
        let mean = opencv::core::mean(&bot_grey, &Mat::default()).expect("Failed to get mean");

        let met = if inv { mean[0] < threshold } else { mean[0] > threshold };
        log::trace!("MEAN = {}", mean[0]);

        let proc = if inv { Processing::USUMBottomScreenInv(threshold) } else { Processing::USUMBottomScreen(threshold) };

        ProcessingResult {
            process: proc,
            met,
            species: 0,
            shiny: false,
        }
    }

    fn process(
        &mut self,
        process: &Processing,
        top_frame: &Mat,
        bot_frame: &Mat,
    ) -> ProcessingResult {
        match process {
            Processing::USUMShinyStar(target) => self.shiny_star(top_frame, *target),
            Processing::USUMBottomScreen(threshold) => self.bottom(bot_frame, *threshold, false),
            Processing::USUMBottomScreenInv(threshold) => self.bottom(bot_frame, *threshold, true),
            _ => unimplemented!("Processing not implemented for 3DS"),
        }
    }
}

impl BishaanVisionSocket {
    pub async fn new(
        ip: core::net::Ipv4Addr,
        tx_top: watch::Sender<Mat>,
        tx_bottom: watch::Sender<Mat>,
        can_heartbeat: Arc<AtomicBool>,
        error_tx: Arc<broadcast::Sender<ShaooohError>>,
    ) -> std::io::Result<Self> {
        log::info!("Creating BishaanVisionSocket");

        let img_socket = UdpSocket::bind("0.0.0.0:8001").await?;
        img_socket.connect((ip.clone(), 8000)).await?;

        {
            let mut ctl1_socket = TcpStream::connect((ip.clone(), 8000)).await?;

            // Send init packet
            let init = NTRPacket::init();
            ctl1_socket.write_all(&init.to_wire()).await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        }

        let tcp_sock = TcpStream::connect((ip, 8000)).await?;

        let last_fps = SystemTime::now();
        let last_frame_count = 0;

        Ok(Self {
            img_socket,
            tcp_sock: Some(tcp_sock),
            tx_top,
            tx_bottom,
            top_frame_num: 0,
            top_frame_seq: 0,
            top_screen_buf: vec![],
            bot_frame_num: 0,
            bot_frame_seq: 0,
            bot_screen_buf: vec![],
            can_heartbeat,
            last_fps,
            last_frame_count,
            error_tx,
        })
    }

    async fn listen(&mut self) -> std::io::Result<Frame> {
        let mut frame = Frame::None;
        let mut buf = [0u8; 1500];
        let n = self.img_socket.recv(&mut buf).await?;

        let is_top_screen = (buf[1] & 0xf) == 1;
        let is_last = (buf[1] & 0xf0) == 0x10;
        let frame_id = buf[0];
        let seq_num = buf[3];

        if seq_num == 0 {
            // New frame
            if is_top_screen {
                self.top_frame_num = frame_id;
                self.top_frame_seq = 0;
                self.top_screen_buf.clear();
                self.top_screen_buf.extend_from_slice(&buf[4..n]);
            } else {
                self.bot_frame_num = frame_id;
                self.bot_frame_seq = 0;
                self.bot_screen_buf.clear();
                self.bot_screen_buf.extend_from_slice(&buf[4..n]);
            }
        } else {
            let (exp_frame_num, next_seq) = if is_top_screen {
                (self.top_frame_num, self.top_frame_seq + 1)
            } else {
                (self.bot_frame_num, self.bot_frame_seq + 1)
            };

            if exp_frame_num == frame_id && seq_num == next_seq {
                if is_top_screen {
                    self.top_frame_seq = seq_num;
                    self.top_screen_buf.extend_from_slice(&buf[4..n]);
                } else {
                    self.bot_frame_seq = seq_num;
                    self.bot_screen_buf.extend_from_slice(&buf[4..n]);
                }

                if is_last {
                    if is_top_screen {
                        if let Ok(s) = opencv::imgcodecs::imdecode(
                            &opencv::core::Vector::from_slice(&self.top_screen_buf),
                            opencv::imgcodecs::IMREAD_COLOR,
                        ) {
                            self.last_frame_count += 1;

                            if self.last_fps.elapsed().expect("Failed to get time")
                                > Duration::from_secs(1)
                            {
                                log::info!("Last FPS: {}", self.last_frame_count);

                                self.last_frame_count = 0;
                                self.last_fps = SystemTime::now();
                            }

                            let mut m2 = Mat::default();
                            opencv::core::rotate(
                                &s,
                                &mut m2,
                                opencv::core::ROTATE_90_COUNTERCLOCKWISE,
                            );
                            frame = Frame::Top(m2);
                        }
                    } else {
                        if let Ok(s) = opencv::imgcodecs::imdecode(
                            &opencv::core::Vector::from_slice(&self.bot_screen_buf),
                            opencv::imgcodecs::IMREAD_COLOR,
                        ) {
                            let mut m2 = Mat::default();
                            opencv::core::rotate(
                                &s,
                                &mut m2,
                                opencv::core::ROTATE_90_COUNTERCLOCKWISE,
                            );
                            frame = Frame::Bottom(m2);
                        }
                    }
                }
            } else {
                // if is_top_screen {
                //     print!("TOP ");
                // } else {
                //     print!("BOT ");
                // }
                //  println!(
                //      "Missing packet? Expected frame{}, got frame{} - exp{},got{}",
                //      exp_frame_num, frame_id, next_seq, seq_num
                // );
                // Poison sequence
                if is_top_screen {
                    self.top_frame_seq = 250;
                } else {
                    self.bot_frame_seq = 250;
                }
            }
        }

        if self.last_fps.elapsed().unwrap() > Duration::from_secs(10) {
            log::error!("Haven't got a new frame in 10 seconds");
            self.error_tx
                .send(ShaooohError::CommunicationError)
                .expect("Failed to send error");
        }

        Ok(frame)
    }

    pub async fn task(mut self) -> std::io::Result<()> {
        let can_heartbeat = self.can_heartbeat.clone();
        let (mut read, mut write) = self
            .tcp_sock
            .take()
            .expect("Failed to get socket")
            .into_split();
        let error_tx = self.error_tx.clone();

        tokio::spawn(async move {
            loop {
                let mut header_buf = [0u8; NTRPacket::HDR_SIZE];
                let r = read.read(&mut header_buf).await;
                match r {
                    Ok(n) => {
                        if n == 84 {
                            if let Some(hdr) = NTRPacket::from_wire(&header_buf) {
                                if hdr.extra_len() > 0 {
                                    let mut extra_buf = vec![0u8; hdr.extra_len()];
                                    let e_res = read.read(&mut extra_buf).await;
                                    match e_res {
                                        Ok(n) => {
                                            let str_conv = String::from_utf8_lossy(&extra_buf);
                                            let strings = str_conv.split('\n');
                                            for s in strings {
                                                if s.len() > 0 {
                                                    log::info!("[NTR] {}", s);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            log::error!("{:?}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("{:?}", e);
                    }
                }
            }
        });
        tokio::spawn(async move {
            let mut seq = 1;
            loop {
                tokio::time::sleep(Duration::from_millis(250)).await;
                let hb_pkt = NTRPacket::heartbeat(seq);
                if can_heartbeat.load(Ordering::Acquire) {
                    match write.write_all(&hb_pkt.to_wire()).await {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Heartbeat send error: {:?}", e);
                            error_tx
                                .send(ShaooohError::CommunicationError)
                                .expect("Failed to send error");
                            break;
                        }
                    }
                    seq += 1;
                }
            }
        });

        loop {
            match tokio::time::timeout(Duration::from_secs(10), self.listen()).await {
                Ok(frame_res) => match frame_res {
                    Ok(frame) => match frame {
                        Frame::None => {}
                        Frame::Bottom(m) => {
                            if self.tx_bottom.send(m).is_err() {
                                break;
                            }
                        }
                        Frame::Top(m) => {
                            if self.tx_top.send(m).is_err() {
                                break;
                            }
                        }
                    },
                    Err(_) => {
                        log::error!("Frame error");
                        self.error_tx
                            .send(ShaooohError::CommunicationError)
                            .expect("Failed to send error");
                        break;
                    }
                },
                Err(_) => {
                    log::error!("Haven't got a new frame in 10 seconds");
                    self.error_tx
                        .send(ShaooohError::CommunicationError)
                        .expect("Failed to send error");
                    break;
                }
            }
        }

        log::info!("Bishaan Vision socket complete");
        Ok(())
    }
}
