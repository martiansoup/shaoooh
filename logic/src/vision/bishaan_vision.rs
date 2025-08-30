use crate::vision::{BotVision, ProcessingResult};

use opencv::{prelude::*, core::Vector};

use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, UdpSocket},
    sync::watch,
};

use super::NTRPacket;

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
}

pub struct BishaanVisionSocket {
    tx_top: watch::Sender<Mat>,
    tx_bottom: watch::Sender<Mat>,
    tcp_sock: TcpStream,
    img_socket: UdpSocket,
    top_frame_num: u8,
    top_frame_seq: u8,
    top_screen_buf: Vec<u8>,
    bot_frame_num: u8,
    bot_frame_seq: u8,
    bot_screen_buf: Vec<u8>,
    heartbeat_seq: u32,
}

impl BotVision for BishaanVision {
    fn process_next_frame(
        &mut self,
        processing: &[super::Processing],
    ) -> Option<Vec<ProcessingResult>> {
        let mut results = vec![];
        for proc in processing {
            results.push(ProcessingResult {
                process: proc.clone(),
                met: true,
                species: 0,
                shiny: true,
            });
        }

        {
            let top = self.rx_top.borrow().clone();
            if !top.empty() {
                opencv::imgcodecs::imencode(".png", &top, &mut self.encoded_top, &Vector::new())
                .expect("Failed to encode frame");
                opencv::highgui::imshow("top", &top)
                    .unwrap_or_else(|_| panic!("Failed to show top window"));
                opencv::highgui::wait_key(1).expect("Event loop failed");
            }
        }
        {
            let bottom = self.rx_bottom.borrow().clone();
            if !bottom.empty() {
                opencv::imgcodecs::imencode(".png", &bottom, &mut self.encoded_bottom, &Vector::new())
                .expect("Failed to encode frame");
                opencv::highgui::imshow("bottom", &bottom)
                    .unwrap_or_else(|_| panic!("Failed to show bottom window"));
                opencv::highgui::wait_key(1).expect("Event loop failed");
            }
        }

        Some(results)
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
        let frame = if let Ok(f) = std::fs::read("static/metamon.png") {
            f
        } else {
            vec![]
        };

        BishaanVision {
            rx_top,
            rx_bottom,
            encoded_top: Vector::default(),
            encoded_bottom: Vector::default()
        }
    }
}

impl BishaanVisionSocket {
    pub async fn new(
        ip: core::net::Ipv4Addr,
        tx_top: watch::Sender<Mat>,
        tx_bottom: watch::Sender<Mat>,
    ) -> std::io::Result<Self> {
        // TODO heartbeat etc.
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
        Ok(Self {
            img_socket,
            tcp_sock,
            tx_top,
            tx_bottom,
            top_frame_num: 0,
            top_frame_seq: 0,
            top_screen_buf: vec![],
            bot_frame_num: 0,
            bot_frame_seq: 0,
            bot_screen_buf: vec![],
            heartbeat_seq: 1,
        })
    }

    async fn listen(&mut self) -> std::io::Result<Frame> {
        let mut frame = Frame::None;
        let mut buf = vec![0; 1500];
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
                // println!(
                //     "Missing packet? Expected frame{}, got frame{} - exp{},got{}",
                //     exp_frame_num, frame_id, next_seq, seq_num
                // );
                // Poison sequence
                if is_top_screen {
                    self.top_frame_seq = 250;
                } else {
                    self.bot_frame_seq = 250;
                }
            }
        }

        Ok(frame)
    }

    pub async fn task(mut self) -> std::io::Result<()> {
        while let Ok(frame) = self.listen().await {
            match frame {
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
            }
        }

        log::info!("Bishaan Vision socket complete");
        Ok(())
    }
}
