use crate::vision::{BotVision, ProcessingResult};

use opencv::prelude::*;

use tokio::{
    net::{TcpStream, UdpSocket},
    sync::watch,
};

pub struct BishaanVision {
    frame: Vec<u8>,
    rx: watch::Receiver<Mat>,
}

pub struct BishaanVisionSocket {
    tx: watch::Sender<Mat>,
    //tcp_sock: TcpStream,
    udp_sock: UdpSocket,
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

        opencv::highgui::wait_key(1).expect("Event loop failed");

        Some(results)
    }

    fn read_frame(&self) -> &[u8] {
        &self.frame
    }
}

impl BishaanVision {
    pub fn new(rx: watch::Receiver<Mat>) -> Self {
        let frame = if let Ok(f) = std::fs::read("static/metamon.png") {
            f
        } else {
            vec![]
        };

        BishaanVision { frame, rx }
    }
}

impl BishaanVisionSocket {
    pub async fn new(ip: core::net::Ipv4Addr, tx: watch::Sender<Mat>) -> std::io::Result<Self> {
        // TODO heartbeat etc.
        let udp_sock = UdpSocket::bind("0.0.0.0:8001").await?;
        //let tcp_sock = TcpStream::connect((ip, 8000)).await?;
        Ok(Self {
            udp_sock,
            //tcp_sock,
            tx,
        })
    }

    pub async fn task(mut self) {
        log::info!("Bishaan Vision socket complete")
    }
}
