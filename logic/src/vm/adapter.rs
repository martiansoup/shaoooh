use crate::vision::Processing;
use crate::vision::collection::AvailableProcessing;

use crate::app::Game;

use crate::control::Button;

use crate::hunt::HuntStateOutput;

pub trait StateParser {
    fn parse(s: &str, game: Game, strict: bool) -> Result<Self, String>
    where
        Self: Sized;
}

impl StateParser for Processing {
    fn parse(s: &str, game: Game, strict: bool) -> Result<Self, String> {
        let mut args = s.split(":");

        let proc = match args.next() {
            Some("Sprite") => {
                let targets = args.next();
                let flipped = args.next();

                if let (Some(targets), Some(flipped)) = (targets, flipped) {
                    let targets = targets.split("+").flat_map(|n| n.parse::<u32>()).collect();
                    let flipped = match flipped {
                        "0" | "false" => false,
                        "1" | "true" => true,
                        _ => false,
                    };
                    Some(Processing::Sprite(game, targets, flipped))
                } else {
                    None
                }
            }
            Some(s) => {
                let procs = AvailableProcessing::new();

                if let Ok(procs) = procs {
                    procs.resolve(s)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(p) = proc {
            return Ok(p);
        }

        if strict {
            Err(format!("No processing match for: {}", s))
        } else {
            Ok(Processing::Input(s.to_string()))
        }
    }
}

impl StateParser for String {
    fn parse(s: &str, _: Game, _: bool) -> Result<Self, String> {
        Ok(s.to_string())
    }
}

impl StateParser for HuntStateOutput {
    fn parse(s: &str, _: Game, _: bool) -> Result<Self, String> {
        match s {
            "A" => Ok(HuntStateOutput::button(Button::A)),
            "B" => Ok(HuntStateOutput::button(Button::B)),
            "X" => Ok(HuntStateOutput::button(Button::X)),
            "Y" => Ok(HuntStateOutput::button(Button::Y)),
            "Start" => Ok(HuntStateOutput::button(Button::Start)),
            "Select" => Ok(HuntStateOutput::button(Button::Select)),
            "L" => Ok(HuntStateOutput::button(Button::L)),
            "R" => Ok(HuntStateOutput::button(Button::R)),
            "Left" => Ok(HuntStateOutput::button(Button::Left)),
            "Right" => Ok(HuntStateOutput::button(Button::Right)),
            "Up" => Ok(HuntStateOutput::button(Button::Up)),
            "Down" => Ok(HuntStateOutput::button(Button::Down)),
            "Home" => Ok(HuntStateOutput::button(Button::Home)),
            "ZL" => Ok(HuntStateOutput::button(Button::ZL)),
            "ZR" => Ok(HuntStateOutput::button(Button::ZR)),
            _ => Err("Failed to parse button".to_string()),
        }
    }
}
