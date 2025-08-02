use serialport::SerialPort;

use crate::app::states::AppState;

pub struct GfxDisplay {
    last_target: u32,
    last_encounters: u64,
    serial_disp: Option<Box<dyn SerialPort>>,
}

impl super::StateReceiver for GfxDisplay {
    fn display(&mut self, state: AppState) {
        if state.encounters != self.last_encounters {
            if let Some(tx) = &mut self.serial_disp {
                let phased = state.encounters;
                let enc_str = format!("E{}e", phased);
                tx.write_all(enc_str.as_bytes())
                    .expect("Failed to write encounters to display");
            };
            self.last_encounters = state.encounters;
        }

        if let Some(arg) = state.arg {
            if arg.species != self.last_target {
                let target = arg.species;
                if let Some(tx) = &mut self.serial_disp {
                    let tgt_str = format!("T{}e", target);
                    log::info!("Setting target on display to {}", tgt_str);
                    tx.write_all(tgt_str.as_bytes())
                        .expect("Failed to write target to display");
                };
                self.last_target = target;
            }
        }
    }
}

impl Default for GfxDisplay {
    fn default() -> Self {
        let serial_disp = serialport::new("/dev/ttyACM0", 115200).open().ok();
        let last_target = 0;
        let last_encounters = 0;
        Self {
            last_target,
            last_encounters,
            serial_disp,
        }
    }
}
