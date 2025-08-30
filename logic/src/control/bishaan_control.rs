use crate::control::{BotControl, Button, Delay};

use tokio::net::UdpSocket;
use tokio::sync::mpsc;

pub struct BishaanControl {
    tx: mpsc::Sender<(Vec<Button>, Delay)>,
}

pub struct BishaanControlSocket {
    socket: UdpSocket,
    rx: mpsc::Receiver<(Vec<Button>, Delay)>,
}

impl BotControl for BishaanControl {
    fn press(&mut self, button: &Button) {
        let vec = vec![button.clone()];
        self.tx
            .blocking_send((vec, Delay::Tenth))
            .expect("Failed to send button");
    }

    fn presses_delay(&mut self, buttons: &[&Button], delay: &Delay) {
        self.tx
            .blocking_send((
                buttons.iter().map(|x| (*x).clone()).collect(),
                delay.clone(),
            ))
            .expect("Failed to send buttons");
    }

    fn press_delay(&mut self, button: &Button, delay: &Delay) {
        let vec = vec![button.clone()];
        self.tx
            .blocking_send((vec, delay.clone()))
            .expect("Failed to send button");
    }
}

impl BishaanControl {
    pub fn new(tx: mpsc::Sender<(Vec<Button>, Delay)>) -> Self {
        log::info!("Creating BishaanControl");
        Self { tx }
    }
}

impl BishaanControlSocket {
    const TOUCH_SCREEN_WIDTH: u32 = 320;
    const TOUCH_SCREEN_HEIGHT: u32 = 240;

    async fn get_socket(ip: core::net::Ipv4Addr) -> std::io::Result<UdpSocket> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        socket.connect((ip, 4950)).await?;

        Ok(socket)
    }

    pub async fn new(
        ip: core::net::Ipv4Addr,
        rx: mpsc::Receiver<(Vec<Button>, Delay)>,
    ) -> std::io::Result<BishaanControlSocket> {
        log::info!("Creating BishaanControlSocket");

        let socket = Self::get_socket(ip).await?;

        Ok(BishaanControlSocket { socket, rx })
    }

    pub async fn task(mut self) -> std::io::Result<()> {
        while let Some((buttons, delay)) = self.rx.recv().await {
            self.socket.send(&Self::get_buf(buttons.as_ref())).await?;
            let duration = tokio::time::Duration::from_millis(match delay {
                Delay::Half => 500,
                Delay::Sec => 1000,
                // Unreliable if delay is too short
                Delay::Tenth => 200,
                Delay::Twentieth => 200,
            });
            tokio::time::sleep(duration).await;
            self.socket.send(&Self::get_buf(&[])).await?;
            tokio::time::sleep(duration).await;
        }
        log::info!("Bishaan Control socket complete");
        Ok(())
    }

    fn get_pad(buttons: &[Button]) -> u32 {
        let mut val = 0xfff;
        val &= !(if buttons.contains(&Button::A) {
            0x1 << 0
        } else {
            0x0
        });
        val &= !(if buttons.contains(&Button::B) {
            0x1 << 1
        } else {
            0x0
        });
        val &= !(if buttons.contains(&Button::Select) {
            0x1 << 2
        } else {
            0x0
        });
        val &= !(if buttons.contains(&Button::Start) {
            0x1 << 3
        } else {
            0x0
        });
        val &= !(if buttons.contains(&Button::Right) {
            0x1 << 4
        } else {
            0x0
        });
        val &= !(if buttons.contains(&Button::Left) {
            0x1 << 5
        } else {
            0x0
        });
        val &= !(if buttons.contains(&Button::Up) {
            0x1 << 6
        } else {
            0x0
        });
        val &= !(if buttons.contains(&Button::Down) {
            0x1 << 7
        } else {
            0x0
        });
        val &= !(if buttons.contains(&Button::R) {
            0x1 << 8
        } else {
            0x0
        });
        val &= !(if buttons.contains(&Button::L) {
            0x1 << 9
        } else {
            0x0
        });
        val &= !(if buttons.contains(&Button::X) {
            0x1 << 10
        } else {
            0x0
        });
        val &= !(if buttons.contains(&Button::Y) {
            0x1 << 11
        } else {
            0x0
        });
        // 0x2 - Power
        // 0x4 - Power(long)
        val
    }

    fn get_intf(buttons: &[Button]) -> u32 {
        let mut val = 0;
        val |= if buttons.contains(&Button::Home) {
            0x1
        } else {
            0x0
        };
        // 0x2 - Power
        // 0x4 - Power(long)
        val
    }

    fn get_cpp(buttons: &[Button]) -> u32 {
        let mut val = 0x80800081;
        // C-stick not implemented
        val |= if buttons.contains(&Button::ZL) {
            0x4 << 8
        } else {
            0x0
        };
        val |= if buttons.contains(&Button::ZR) {
            0x2 << 8
        } else {
            0x0
        };
        val
    }

    fn get_circle(buttons: &[Button]) -> u32 {
        let val = buttons
            .iter()
            .find(|b| match b {
                Button::Circle(_, _) => true,
                _ => false,
            })
            .map_or(0x7ff7ff, |b| match b {
                Button::Circle(x, y) => {
                    let mult: u32 = 16;
                    let x32: u32 = (*x as u32) * mult;
                    let y32: u32 = (*y as u32) * mult;
                    x32 | (y32 << 12)
                }
                _ => unreachable!("Filtered to circle"),
            });

        val
    }

    fn get_touch(buttons: &[Button]) -> u32 {
        buttons
            .iter()
            .find(|b| match b {
                Button::Touch(_, _) => true,
                _ => false,
            })
            .map_or(0x2000000, |b| match b {
                Button::Touch(x, y) => {
                    let x32: u32 = (0xfff * std::cmp::min(*x as u32, Self::TOUCH_SCREEN_WIDTH))
                        / Self::TOUCH_SCREEN_WIDTH;
                    let y32: u32 = (0xfff * std::cmp::min(*y as u32, Self::TOUCH_SCREEN_HEIGHT))
                        / Self::TOUCH_SCREEN_HEIGHT;
                    (1 << 24) | (y32 << 12) | x32
                }
                _ => unreachable!("Filtered to touch"),
            })
    }

    fn get_buf(buttons: &[Button]) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        let pad: u32 = Self::get_pad(buttons);
        buf.extend_from_slice(&pad.to_le_bytes());

        let touch_screen: u32 = Self::get_touch(buttons);
        buf.extend_from_slice(&touch_screen.to_le_bytes());

        let circle: u32 = Self::get_circle(buttons);
        buf.extend_from_slice(&circle.to_le_bytes());

        let cpp: u32 = Self::get_cpp(buttons);
        buf.extend_from_slice(&cpp.to_le_bytes());

        let interface: u32 = Self::get_intf(buttons);
        buf.extend_from_slice(&interface.to_le_bytes());

        debug_assert_eq!(buf.len(), 20, "Length must be 20");

        buf
    }
}
