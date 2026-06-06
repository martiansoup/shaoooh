use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::vision::Processing;

#[derive(Serialize, Deserialize)]
pub struct AvailableProcessing {
    builtins: HashMap<String, Processing>,
    custom: HashMap<String, Processing>,
}

impl AvailableProcessing {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let builtins = HashMap::new();
        let custom = HashMap::new();

        AvailableProcessing { builtins, custom }
            .with_builtins()
            .with_custom()
    }

    fn with_builtins(mut self) -> Self {
        self.add_builtin("BW_HP_BAR_PRESENT", Processing::BW_HP_BAR_PRESENT);
        self.add_builtin("BW_BALL_ANIMATION", Processing::BW_BALL_ANIMATION);
        self.add_builtin("BW2_BLACK_SCREEN", Processing::BW2_BLACK_SCREEN);
        self.add_builtin("BW2_WHITE_SCREEN", Processing::BW2_WHITE_SCREEN);
        self.add_builtin("BW2_BAR_PRESENT", Processing::BW2_BAR_PRESENT);
        self.add_builtin("BW2_BAR_NEGATE_CONFIRM", Processing::BW2_BAR_NEGATE_CONFIRM);
        self.add_builtin(
            "DP_START_ENCOUNTER_WHITE",
            Processing::DP_START_ENCOUNTER_WHITE,
        );
        self.add_builtin("DP_START_ENCOUNTER", Processing::DP_START_ENCOUNTER);
        self.add_builtin("HGSS_BLACK_SCREEN", Processing::HGSS_BLACK_SCREEN);
        self.add_builtin("DP_IN_ENCOUNTER", Processing::DP_IN_ENCOUNTER);
        self.add_builtin("DP_ENCOUNTER_READY", Processing::DP_ENCOUNTER_READY);
        self.add_builtin("HGSS_ENCOUNTER_READY", Processing::HGSS_ENCOUNTER_READY);
        self.add_builtin("HGSS_BATTLE_BLACK_BAR", Processing::HGSS_BATTLE_BLACK_BAR);
        self.add_builtin("HGSS_BATTLE_WHITE_TOP", Processing::HGSS_BATTLE_WHITE_TOP);
        self.add_builtin(
            "DP_SAFARI_ENCOUNTER_READY",
            Processing::DP_SAFARI_ENCOUNTER_READY,
        );
        self.add_builtin("FRLG_SHINY_STAR", Processing::FRLG_SHINY_STAR);
        self.add_builtin("USUM_SHINY_STAR", Processing::USUM_SHINY_STAR);
        self.add_builtin("FRLG_SHINY_STAR_OLD", Processing::FRLG_SHINY_STAR_OLD);
        self.add_builtin("FRLG_START_ENCOUNTER", Processing::FRLG_START_ENCOUNTER);
        self.add_builtin("FRLG_IN_ENCOUNTER", Processing::FRLG_IN_ENCOUNTER);
        self.add_builtin("FRLG_ENCOUNTER_READY", Processing::FRLG_ENCOUNTER_READY);
        self.add_builtin("RS_FISHING_ACTIVE", Processing::RS_FISHING_ACTIVE);
        self.add_builtin("RS_FISHING_BITE", Processing::RS_FISHING_BITE);
        self.add_builtin("RS_FISHING_ON_HOOK", Processing::RS_FISHING_ON_HOOK);
        self.add_builtin("RS_FISHING_NO_NIBBLE", Processing::RS_FISHING_NO_NIBBLE);

        self
    }

    fn add_builtin(&mut self, s: &str, p: Processing) {
        self.builtins.insert(s.to_string(), p);
    }

    fn with_custom(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let custom_file = std::fs::File::open("fsm/processing.json")?;
        let reader = std::io::BufReader::new(custom_file);

        let custom: HashMap<String, Processing> = serde_json::from_reader(reader)?;

        self.custom.extend(custom);

        Ok(self)
    }

    pub fn resolve(&self, s: &str) -> Option<Processing> {
        let (key, to_search) = if s.starts_with('$') {
            (s.strip_prefix('$').unwrap(), &self.builtins)
        } else {
            (s, &self.custom)
        };

        to_search.get(key).cloned()
    }
}
