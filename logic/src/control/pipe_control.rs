use crate::control::{BotControl, Button, Delay};
use tokio::net::unix::pipe;
use tokio::sync::mpsc;

use tokio::io::AsyncWriteExt;

pub struct GyaaasControlSocket {
    pipe: tokio::net::unix::pipe::Sender,
    rx: mpsc::Receiver<(Vec<Button>, Delay)>,
}

impl GyaaasControlSocket {
    pub fn new(path: &str, rx: mpsc::Receiver<(Vec<Button>, Delay)>) -> Self {
        let pipe = pipe::OpenOptions::new()
            .open_sender(path)
            .expect("Failed to open control fifo");
        GyaaasControlSocket { pipe, rx }
    }

    pub async fn task(mut self) -> std::io::Result<()> {
        while let Some((buttons, delay)) = self.rx.recv().await {
            let control_string = if buttons.contains(&Button::A)
                && buttons.contains(&Button::B)
                && buttons.contains(&Button::Select)
                && buttons.contains(&Button::Start)
            {
                format!("q!1qpq!0")
            } else {
                let mut control_string = "".to_string();
                for b in buttons.iter().map(|x| (*x).clone()) {
                    control_string += &Self::get_button_str(&b, true);
                }
                control_string += &Self::get_delay_str(&delay);
                for b in buttons.iter().map(|x| (*x).clone()) {
                    control_string += &Self::get_button_str(&b, false);
                }
                control_string
            };

            self.pipe
                .write_all(control_string.as_bytes())
                .await
                .expect("Couldn't write");
        }
        log::info!("Gyaaas Control socket complete");
        Ok(())
    }

    fn get_button_str(button: &Button, down: bool) -> String {
        let cchar = match button {
            Button::A => Some('A'),
            Button::B => Some('B'),
            Button::X => Some('X'),
            Button::Y => Some('Y'),
            Button::Start => Some('S'),
            Button::Select => Some('s'),
            Button::L => Some('L'),
            Button::R => Some('R'),
            Button::Left => Some('l'),
            Button::Right => Some('r'),
            Button::Up => Some('u'),
            Button::Down => Some('d'),
            Button::Home => Some('h'),
            Button::Circle(..) | Button::Touch(..) | Button::ZL | Button::ZR => {
                log::warn!("Use of unsupported button: {:?}", button);
                None
            }
        };
        if let Some(c) = cchar {
            let val_char = if down { "1" } else { "0" };
            format!("q{}{}", c, val_char)
        } else {
            "".to_string()
        }
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
}
