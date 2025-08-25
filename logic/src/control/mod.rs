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

#[allow(dead_code)] // TODO are Half/Sec needed?
#[derive(PartialEq, Debug, Clone)]
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

    pub fn press(&mut self, button: &Button) {
        self.press_delay(button, &Delay::Tenth);
    }

    fn get_button_str(button: &Button, down: bool) -> String {
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
        let val_char = if down { "1" } else { "0" };
        format!("q{}{}", cchar, val_char)
    }

    fn get_delay_str(delay: &Delay) -> String {
        let pchar = match delay {
            Delay::Half => 'P',
            Delay::Sec => 'M',
            Delay::Tenth => 'p',
            Delay::Twentieth => 'm',
        };
        format!("q{}", pchar)
    }

    pub fn presses_delay(&mut self, buttons: &[&Button], delay: &Delay) {
        let mut control_string = "".to_string();
        for b in buttons {
            control_string += &Self::get_button_str(b, true);
        }
        control_string += &Self::get_delay_str(delay);
        for b in buttons {
            control_string += &Self::get_button_str(b, false);
        }
        self.port
            .write_all(control_string.as_bytes())
            .expect("Couldn't write");
    }

    pub fn press_delay(&mut self, button: &Button, delay: &Delay) {
        let down = Self::get_button_str(button, true);
        let pause = Self::get_delay_str(delay);
        let up = Self::get_button_str(button, false);
        let control_string = format!("{}{}{}", down, pause, up);
        self.port
            .write_all(control_string.as_bytes())
            .expect("Couldn't write");
    }

    pub fn gen3_soft_reset(&mut self) {
        self.presses_delay(
            &[&Button::A, &Button::B, &Button::Start, &Button::Select],
            &Delay::Tenth,
        );
    }

    pub fn gen5_soft_reset(&mut self) {
        self.presses_delay(
            &[&Button::L, &Button::R, &Button::Start, &Button::Select],
            &Delay::Tenth,
        );
    }
}

impl Default for ShaooohControl {
    fn default() -> Self {
        Self::new()
    }
}
