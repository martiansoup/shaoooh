use serde::{Deserialize, Serialize};
use serialport::SerialPort;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Button {
    A,
    B,
    X,
    Y,
    Start,
    Select,
    L,
    R,
    Left,
    Right,
    Up,
    Down,
}

pub enum Delay {
    Twentieth,
    Tenth,
    Half,
    Sec,
}

pub struct ShaooohControl {
    port: Box<dyn SerialPort>,
}

impl ShaooohControl {
    pub fn new() -> ShaooohControl {
        log::info!("Connecting to serial port");
        ShaooohControl {
            port: serialport::new("/dev/ttyAMA0", 115200)
                .open()
                .expect("Unable to open"),
        }
    }

    // TODO add delay
    pub fn press(&mut self, button: Button) {
        self.press_delay(button, Delay::Tenth);
    }

    pub fn press_delay(&mut self, button: Button, delay: Delay) {
        let cchar = match button {
            Button::A => 'A',
            Button::B => 'B',
            Button::X => 'X',
            Button::Y => 'Y',
            Button::Start => 'S',
            Button::Select => 's',
            Button::L => 'L',
            Button::R => 'R',
            Button::Left => 'l',
            Button::Right => 'r',
            Button::Up => 'u',
            Button::Down => 'd',
        };
        let pchar = match delay {
            Delay::Half => 'P',
            Delay::Sec => 'M',
            Delay::Tenth => 'p',
            Delay::Twentieth => 'm',
        };
        let control_string = format!("q{}1q{}q{}0", cchar, pchar, cchar);
        self.port
            .write_all(control_string.as_bytes())
            .expect("Couldn't write");
    }

    pub fn gen3_soft_reset(&mut self) {
        let control_string = "qA1qB1qS1qs1qpqA0qB0qS0qs0";
        self.port
            .write_all(control_string.as_bytes())
            .expect("Couldn't write");
    }
}
