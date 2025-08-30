use serde::{Deserialize, Serialize};

mod bishaan_control;
mod nop_control;
mod serial_control;

pub use bishaan_control::{BishaanControl, BishaanControlSocket};
pub use nop_control::NopControl;
pub use serial_control::ShaooohControl;

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
    Home,
    ZL,
    ZR,
    Circle(u8, u8),
    Touch(u16, u16),
}

#[allow(dead_code)] // TODO are Half/Sec needed?
#[derive(PartialEq, Debug, Clone)]
pub enum Delay {
    Twentieth,
    Tenth,
    Half,
    Sec,
}

pub trait BotControl {
    fn press(&mut self, button: &Button);
    fn presses_delay(&mut self, buttons: &[&Button], delay: &Delay);
    fn press_delay(&mut self, button: &Button, delay: &Delay);
}
