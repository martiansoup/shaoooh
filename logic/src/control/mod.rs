use serde::{Serialize, Deserialize};
use serialport::{SerialPort, TTYPort};

#[derive(Clone, Serialize, Deserialize, Debug)]
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
    Down
}

pub struct ShaooohControl {
    port: Box<dyn SerialPort>
}

impl ShaooohControl {
    pub fn new() -> ShaooohControl {
        log::info!("Connecting to serial port");
        ShaooohControl {
            port: serialport::new("/dev/ttyAMA0", 115200).open().expect("Unable to open")
        }
    }

    pub fn press(&mut self, button: Button) {
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
            Button::Down => 'd'
        };
        let control_string = format!("q{}1qpq{}0", cchar, cchar);
        // TODO check number of bytes written and write remaining if needed
        self.port.write(control_string.as_bytes()).expect("Couldn't write");
    }

}