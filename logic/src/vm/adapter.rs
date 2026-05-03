use crate::vision::Processing;

use crate::app::Game;

pub trait StateParser {

    fn parse(s: &str, game: Game, strict: bool) -> Result<Self, String> where Self: Sized;
}

impl StateParser for Processing {

    fn parse(s: &str, game: Game, strict: bool) -> Result<Self, String> {
        let mut args = s.split(":");

        let proc = match args.next() {
            Some("Sprite") => {
                let targets = args.next();
                let flipped = args.next();

                if let (Some(targets), Some(flipped)) = (targets, flipped) {
                    let targets = targets.split("+").map(|n| n.parse::<u32>()).flatten().collect();
                    let flipped = match flipped {
                        "0" | "false" => false,
                        "1" | "true" => true,
                        _ => false,
                    };
                    Some(Processing::Sprite(game, targets, flipped))
                } else {
                    None
                }
            },
            Some("BW_HP_BAR_PRESENT") => Some(Processing::BW_HP_BAR_PRESENT),
            Some("BW_BALL_ANIMATION") => Some(Processing::BW_BALL_ANIMATION),
            Some("BW2_BLACK_SCREEN") => Some(Processing::BW2_BLACK_SCREEN),
            Some("BW2_WHITE_SCREEN") => Some(Processing::BW2_WHITE_SCREEN),
            Some("BW2_BAR_PRESENT") => Some(Processing::BW2_BAR_PRESENT),
            Some("BW2_BAR_NEGATE_CONFIRM") => Some(Processing::BW2_BAR_NEGATE_CONFIRM),
            Some("DP_START_ENCOUNTER_WHITE") => Some(Processing::DP_START_ENCOUNTER_WHITE),
            Some("DP_START_ENCOUNTER") => Some(Processing::DP_START_ENCOUNTER),
            Some("HGSS_BLACK_SCREEN") => Some(Processing::HGSS_BLACK_SCREEN),
            Some("DP_IN_ENCOUNTER") => Some(Processing::DP_IN_ENCOUNTER),
            Some("DP_ENCOUNTER_READY") => Some(Processing::DP_ENCOUNTER_READY),
            Some("HGSS_ENCOUNTER_READY") => Some(Processing::HGSS_ENCOUNTER_READY),
            Some("DP_SAFARI_ENCOUNTER_READY") => Some(Processing::DP_SAFARI_ENCOUNTER_READY),
            Some("FRLG_SHINY_STAR") => Some(Processing::FRLG_SHINY_STAR),
            Some("USUM_SHINY_STAR") => Some(Processing::USUM_SHINY_STAR),
            Some("FRLG_SHINY_STAR_OLD") => Some(Processing::FRLG_SHINY_STAR_OLD),
            Some("FRLG_START_ENCOUNTER") => Some(Processing::FRLG_START_ENCOUNTER),
            Some("FRLG_IN_ENCOUNTER") => Some(Processing::FRLG_IN_ENCOUNTER),
            Some("FRLG_ENCOUNTER_READY") => Some(Processing::FRLG_ENCOUNTER_READY),
            Some("RS_FISHING_ACTIVE") => Some(Processing::RS_FISHING_ACTIVE),
            Some("RS_FISHING_BITE") => Some(Processing::RS_FISHING_BITE),
            Some("RS_FISHING_ON_HOOK") => Some(Processing::RS_FISHING_ON_HOOK),
            Some("RS_FISHING_NO_NIBBLE") => Some(Processing::RS_FISHING_NO_NIBBLE),
            _ => None
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