use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime};

use rand::Rng;

use crate::app::states::{Game, Method, RequestTransition, Transition, TransitionArg};
use crate::control::{Button, Delay, ShaooohControl};
use crate::hunt::{BaseHunt, HuntFSM, HuntResult};
use crate::vision::{Processing, ProcessingResult};

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum RSSoftResetState {
    Init,
    SoftReset,
    StartToFileSelect,
    StartToFileSelect2,
    StartToFileSelect3,
    AToContinue,
    StartEncounter,
    WaitStartEncounter,
    EnteringEncounter,
    WaitEncounterReady,
    Detect,
    Done,
    Wait(Duration, Box<RSSoftResetState>),
}

const SHINY_DURATION: Duration = Duration::from_millis(2400); // TODO timed for regirock

pub(crate) struct RSSoftReset {
    pub(crate) base: BaseHunt,
    pub(crate) state: RSSoftResetState,
    pub(crate) timer: SystemTime,
    pub(crate) last_timer_duration: Duration,
    //pub(crate) stats_file: File,
}

impl RSSoftReset {
    fn create_wait_secs(
        &mut self,
        d: u64,
        state: RSSoftResetState,
    ) -> RSSoftResetState {
        self.base.wait_start = SystemTime::now();
        RSSoftResetState::Wait(Duration::from_secs(d), Box::new(state))
    }

    fn create_wait_msecs(
        &mut self,
        d: u64,
        state: RSSoftResetState,
    ) -> RSSoftResetState {
        self.base.wait_start = SystemTime::now();
        RSSoftResetState::Wait(Duration::from_millis(d), Box::new(state))
    }
}

impl HuntFSM for RSSoftReset {
    fn processing(&self) -> Vec<Processing> {
        if self.state == RSSoftResetState::Detect {
            vec![Processing::Sprite(
                Game::RubySapphire,
                vec![self.base.target],
                false,
            )]
        } else if self.state == RSSoftResetState::WaitStartEncounter {
            vec![Processing::FRLG_START_ENCOUNTER]
        } else if self.state == RSSoftResetState::EnteringEncounter {
            vec![Processing::FRLG_IN_ENCOUNTER]
        } else if self.state == RSSoftResetState::WaitEncounterReady {
            vec![
                Processing::FRLG_IN_ENCOUNTER,
                Processing::FRLG_ENCOUNTER_READY,
            ]
        } else {
            Vec::new()
        }
    }

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult {
        let incr_encounters = self.state == RSSoftResetState::Detect;
        let mut transition = None;
        let mut detect_result = None;
        let mut enter_encounter = false;
        let mut in_encounter = false;
        let mut encounter_ready = false;

        for r in results {
            match r.process {
                Processing::Sprite(_, _, _) => detect_result = Some(r),
                Processing::FRLG_START_ENCOUNTER => enter_encounter = r.met,
                Processing::FRLG_IN_ENCOUNTER => in_encounter = r.met,
                Processing::FRLG_ENCOUNTER_READY => encounter_ready = r.met,
                _ => {}
            }
        }

        let old_state = self.state.clone();

        self.state = match &self.state {
            RSSoftResetState::Init => {
                RSSoftResetState::SoftReset
            }
            RSSoftResetState::SoftReset => {
                control.gen3_soft_reset();
                let mut rng = rand::rng();
                let delay = 5000 + rng.random_range(0..1000);
                self.create_wait_msecs(delay, RSSoftResetState::StartToFileSelect)
            }
            RSSoftResetState::StartToFileSelect => {
                control.press(Button::Start);
                let delay = 5000;
                self.create_wait_msecs(delay, RSSoftResetState::StartToFileSelect2)
            }
            RSSoftResetState::StartToFileSelect2 => {
                control.press(Button::Start);
                let delay = 2000;
                self.create_wait_msecs(delay, RSSoftResetState::StartToFileSelect3)
            }
            RSSoftResetState::StartToFileSelect3 => {
                control.press(Button::Start);
                let delay = 1000;
                self.create_wait_msecs(delay, RSSoftResetState::AToContinue)
            }
            RSSoftResetState::AToContinue => {
                control.press(Button::A);
                let mut rng = rand::rng();
                let delay = 1000 + rng.random_range(0..2100);
                self.create_wait_msecs(delay, RSSoftResetState::StartEncounter)
            }
            RSSoftResetState::StartEncounter => {
                control.press(Button::A);
                let mut rng = rand::rng();
                let delay = 1000 + rng.random_range(0..2100);
                self.create_wait_msecs(delay, RSSoftResetState::WaitStartEncounter)
            }
            RSSoftResetState::WaitStartEncounter => {
                if enter_encounter {
                    RSSoftResetState::EnteringEncounter
                } else {
                    RSSoftResetState::WaitStartEncounter
                }
            }
            RSSoftResetState::EnteringEncounter => {
                if in_encounter {
                    self.timer = SystemTime::now();
                    RSSoftResetState::WaitEncounterReady
                } else {
                    RSSoftResetState::EnteringEncounter
                }
            }
            RSSoftResetState::WaitEncounterReady => {
                if encounter_ready {
                    self.last_timer_duration = self.timer.elapsed().unwrap();
                    RSSoftResetState::Detect
                } else {
                    RSSoftResetState::WaitEncounterReady
                }
            }
            RSSoftResetState::Detect => {
                if let Some(detect) = detect_result {
                    // log::error!("Need to remove stats file");
                    // self.stats_file.write_all(
                    //     format!(
                    //         "species={},time={},shiny={}\n",
                    //         detect.species,
                    //         self.last_timer_duration.as_millis(),
                    //         detect.shiny
                    //     )
                    //     .as_bytes(),
                    // );
                    log::info!("sprite = {}, duration = {:?}", detect.shiny, self.last_timer_duration);
                    if detect.shiny || (self.last_timer_duration > SHINY_DURATION) {
                        if detect.species == self.base.target {
                            transition = Some(RequestTransition {
                                transition: Transition::FoundTarget,
                                arg: None,
                            });
                        } else {
                            transition = Some(RequestTransition {
                                transition: Transition::FoundNonTarget,
                                arg: Some(TransitionArg {
                                    name: String::from(""),
                                    species: detect.species,
                                    game: Game::RubySapphire,
                                    method: Method::SoftResetEncounter,
                                }),
                            });
                        }
                        RSSoftResetState::Done
                    } else {
                        let mut rng = rand::rng();
                        let delay = rng.random_range(0..2000);
                        self.create_wait_msecs(delay, RSSoftResetState::SoftReset)
                    }
                } else {
                    log::error!("No detect result found");
                    RSSoftResetState::Done
                }
            }
            RSSoftResetState::Done => RSSoftResetState::Done,
            RSSoftResetState::Wait(duration, next) => {
                if self.base.wait_start.elapsed().expect("Failed to get time") > *duration {
                    (**next).clone()
                } else {
                    self.state.clone()
                }
            }
        };

        // TODO don't print waits
        if old_state != self.state {
            log::debug!("STATE = {:?} -> {:?}", old_state, self.state);
        }

        HuntResult {
            transition,
            incr_encounters,
        }
    }

    fn cleanup(&mut self) {
        log::info!("Closing FSM");
    }
}
