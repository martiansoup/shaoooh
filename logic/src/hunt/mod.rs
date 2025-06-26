use crate::control::Button;
use std::fs::File;
use std::time::Duration;
use std::time::SystemTime;

mod frlg_starter;
use crate::hunt::frlg_starter::*;
mod frlg_random;
use crate::hunt::frlg_random::*;
mod dp_random;
use crate::hunt::dp_random::*;
mod rs_safari;
use crate::hunt::rs_safari::*;
mod frlg_safari;
use crate::hunt::frlg_safari::*;
mod hgss_safari;
use crate::hunt::hgss_safari::*;

use crate::{
    app::states::{Game, Method, RequestTransition},
    control::ShaooohControl,
    vision::{Processing, ProcessingResult},
};

pub struct HuntResult {
    pub(crate) transition: Option<RequestTransition>,
    pub(crate) incr_encounters: bool,
}

pub trait HuntFSM {
    // TODO methods to be able to draw FSM status

    fn processing(&self) -> Vec<Processing>;

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult;

    fn cleanup(&mut self) {}
}

pub struct HuntBuild {}

impl HuntBuild {
    pub fn build(target: u32, game: Game, method: Method) -> Result<Box<dyn HuntFSM>, ()> {
        let base = BaseHunt {
            target,
            game: game.clone(),
            method: method.clone(),
            wait_start: SystemTime::now(),
        };
        // TODO new method for FSMs
        if game == Game::FireRedLeafGreen
            && method == Method::SoftResetGift
            && (target == 1 || target == 4 || target == 7)
        {
            Ok(Box::new(FRLGStarterGift {
                base,
                state: FRLGStarterGiftState::SoftReset,
            }))
        } else if game == Game::FireRedLeafGreen
            && method == Method::RandomEncounter
            && (target == 16 || target == 19)
        {
            Ok(Box::new(FRLGRandomEncounter {
                base,
                state: FRLGRandomEncounterState::TryGetEncounter,
                next_dir: Button::Down,
                timer: SystemTime::now(),
                last_timer_duration: Duration::default(),
                stats_file: File::create("stats.csv").unwrap(),
            }))
        } else if game == Game::RubySapphire && method == Method::SafariZone && (target == 43) {
            // TODO only oddish in safari zone 1
            Ok(Box::new(RSSafariEncounter {
                base,
                state: RSSafariEncounterState::TryGetEncounter,
                next_dir: Button::Down,
                timer: SystemTime::now(),
                last_timer_duration: Duration::default(),
            }))
        } else if game == Game::FireRedLeafGreen && method == Method::SafariZone && (target == 102) {
            // TODO only Egss in safari zone 3
            Ok(Box::new(FRLGSafariEncounter {
                base,
                state: FRLGSafariEncounterState::TryGetEncounter,
                next_dir: Button::Down,
                timer: SystemTime::now(),
                last_timer_duration: Duration::default(),
            }))
        } else if game == Game::HeartGoldSoulSilver
            && method == Method::SafariZone
            && target == 19
        {
            // TODO only for safari mountain for testing
            Ok(Box::new(HGSSSafariEncounter {
                base,
                state: HGSSSafariEncounterState::Start,
                next_dir: Button::Up,
                timer: SystemTime::now(),
                last_timer_duration: Duration::default(),
                min_shiny: Duration::from_secs(9999),
                min_normal: Duration::from_secs(9999),
                max_shiny: Duration::from_secs(0),
                max_normal: Duration::from_secs(0),
            }))
        } else if game == Game::DiamondPearl
            && method == Method::RandomEncounter
            && (target == 396 || target == 399 || target == 401 || target == 403)
        {
            // TODO only for route 202 for testing
            Ok(Box::new(DPRandomEncounter {
                base,
                state: DPRandomEncounterState::TryGetEncounter,
                next_dir: Button::Up,
                timer: SystemTime::now(),
                last_timer_duration: Duration::default(),
                min_shiny: Duration::from_secs(9999),
                min_normal: Duration::from_secs(9999),
                max_shiny: Duration::from_secs(0),
                max_normal: Duration::from_secs(0),
            }))
        } else {
            log::error!(
                "Hunt not found for target:{}, game:{:?}, method:{:?}",
                target,
                game,
                method
            );
            Err(())
        }
    }
}

struct BaseHunt {
    target: u32,
    game: Game,
    method: Method,
    wait_start: SystemTime,
}
