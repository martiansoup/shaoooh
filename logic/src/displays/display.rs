use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use opencv::core::{Mat, MatTraitConst, Vec3b};
use serialport::SerialPort;

pub struct ScreenDisplay {
    frame_copy: Mat,
    frame_mutex: Arc<Mutex<Mat>>,
    serial_disp: Option<Box<dyn SerialPort>>,
    frame_timer: SystemTime,
    modulo: i32,
}

impl ScreenDisplay {
    //const FRAME_TIME : Duration = Duration::from_millis(1000 / 20);
    const FRAME_TIME: Duration = Duration::from_millis(1000 / 3);
    const WIDTH: i32 = 256;
    const HEIGHT: i32 = 192;
    const CHUNK_SIZE: i32 = 32;

    pub fn new(mutex: Arc<Mutex<Mat>>) -> Self {
        let mut serial_disp = serialport::new("/dev/ttyACM1", 921600).open().ok();
        if let Some(serial) = &mut serial_disp {
            log::info!("Got serial port for display");
            serial
                .clear(serialport::ClearBuffer::All)
                .expect("Failed to clear buffers");
            serial
                .set_timeout(Duration::from_millis(1000))
                .expect("Failed to set timeout");
        } else {
            log::warn!("Failed to get serial port for display");
        }
        let frame_copy = Mat::default();
        ScreenDisplay {
            frame_copy,
            frame_mutex: mutex,
            serial_disp,
            frame_timer: SystemTime::now(),
            modulo: 0,
        }
    }
}

impl super::StateReceiver for ScreenDisplay {
    fn display(&mut self, _state: crate::app::AppState) {
        if self.frame_timer.elapsed().expect("Failed to get time") > Self::FRAME_TIME {
            if let Ok(mat) = self.frame_mutex.lock() {
                self.frame_copy = mat.clone();
            }

            if (self.frame_copy.rows() == Self::HEIGHT) && (self.frame_copy.cols() == Self::WIDTH) {
                for chky in 0..(Self::HEIGHT / Self::CHUNK_SIZE) {
                    for chkx in 0..(Self::WIDTH / Self::CHUNK_SIZE) {
                        let chunk_num = (chky * (Self::WIDTH / Self::CHUNK_SIZE)) + chkx;
                        let num_to_draw = 2;
                        if self.modulo % (48 / num_to_draw) == (chunk_num / num_to_draw) {
                            let mut bytes = Vec::new();
                            bytes.push(0x44);
                            let coord = (chkx & 0xf) | ((chky & 0xf) << 4);
                            let xoff = chkx * Self::CHUNK_SIZE;
                            let yoff = chky * Self::CHUNK_SIZE;
                            bytes.push(coord.try_into().unwrap());
                            for y in 0..Self::CHUNK_SIZE {
                                for x in 0..Self::CHUNK_SIZE {
                                    let a: Vec3b = *self
                                        .frame_copy
                                        .at_2d(y + yoff, x + xoff)
                                        .expect("Couldn't get pixel");
                                    let r = (a.0[2] >> 5) & 0x7;
                                    let g = (a.0[1] >> 5) & 0x7;
                                    let b = (a.0[0] >> 6) & 0x3;
                                    let pixel = (r << 5) | (g << 2) | b;
                                    bytes.push(pixel);
                                }
                            }
                            if let Some(tx) = &mut self.serial_disp {
                                tx.write_all(&bytes).expect("Failed to write to display");
                            }
                        }
                    }
                }
                self.modulo += 1;
            }

            self.frame_timer = SystemTime::now();
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    fn cleanup(&mut self) {}

    fn always_run(&self) -> bool {
        true
    }
}
