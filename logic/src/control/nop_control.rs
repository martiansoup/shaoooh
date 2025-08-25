use crate::control::{BotControl, Button, Delay};

pub struct NopControl {}

impl BotControl for NopControl {
    fn press(&mut self, button: &Button) {
        log::info!("Button press: {:?}", button);
    }

    fn presses_delay(&mut self, buttons: &[&Button], delay: &Delay) {
        log::info!("Button presses: {:?} {:?}", buttons, delay);
    }

    fn press_delay(&mut self, button: &Button, delay: &Delay) {
        log::info!("Button press: {:?} {:?}", button, delay);
    }
}

impl NopControl {
    pub fn new() -> NopControl {
        log::info!("Creating NopControl");
        NopControl {}
    }
}

impl Default for NopControl {
    fn default() -> Self {
        NopControl::new()
    }
}
