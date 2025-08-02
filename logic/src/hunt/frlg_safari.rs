use std::time::{Duration, SystemTime};

use rand::Rng;

use crate::app::states::{Game, Method, RequestTransition, Transition, TransitionArg};
use crate::control::{Button, Delay, ShaooohControl};
use crate::hunt::{BaseHunt, HuntFSM, HuntResult};
use crate::vision::{Processing, ProcessingResult};

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum FRLGSafariEncounterState {
    TryGetEncounter,
    EnteringEncounter,
    WaitEncounterReady,
    PressA,
    Detect,
    Run1Down,
    Run2Right,
    Run3A,
    Run4A,
    Done,
    Wait(Duration, Box<FRLGSafariEncounterState>),
}

// TODO untested
const SHINY_DURATION: Duration = Duration::from_millis(2900);

pub(crate) struct FRLGSafariEncounter {
    pub(crate) base: BaseHunt,
    pub(crate) state: FRLGSafariEncounterState,
    pub(crate) next_dir: Button,
    pub(crate) timer: SystemTime,
    pub(crate) last_timer_duration: Duration,
    //pub(crate) stats_file: File,
}

impl FRLGSafariEncounter {
    fn create_wait_secs(
        &mut self,
        d: u64,
        state: FRLGSafariEncounterState,
    ) -> FRLGSafariEncounterState {
        self.base.wait_start = SystemTime::now();
        FRLGSafariEncounterState::Wait(Duration::from_secs(d), Box::new(state))
    }

    fn create_wait_msecs(
        &mut self,
        d: u64,
        state: FRLGSafariEncounterState,
    ) -> FRLGSafariEncounterState {
        self.base.wait_start = SystemTime::now();
        FRLGSafariEncounterState::Wait(Duration::from_millis(d), Box::new(state))
    }
}

impl HuntFSM for FRLGSafariEncounter {
    fn processing(&self) -> Vec<Processing> {
        if self.state == FRLGSafariEncounterState::Detect {
            // TODO hardcoded list for safari zone area 3
            vec![Processing::Sprite(
                Game::FireRedLeafGreen,
                vec![32, 102, 111, 46, 33, 30, 49, 113, 128],
                false,
            )]
        } else if self.state == FRLGSafariEncounterState::TryGetEncounter {
            vec![Processing::FRLG_START_ENCOUNTER]
        } else if self.state == FRLGSafariEncounterState::EnteringEncounter {
            vec![Processing::FRLG_IN_ENCOUNTER]
        } else if self.state == FRLGSafariEncounterState::WaitEncounterReady {
            vec![
                Processing::FRLG_IN_ENCOUNTER,
                Processing::FRLG_ENCOUNTER_READY,
            ]
        } else {
            Vec::new()
        }
    }

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult {
        let incr_encounters = self.state == FRLGSafariEncounterState::Detect;
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
            FRLGSafariEncounterState::TryGetEncounter => {
                if enter_encounter {
                    self.next_dir = match self.next_dir {
                        Button::Up => Button::Left,
                        Button::Down => Button::Left,
                        Button::Left => Button::Up,
                        Button::Right => Button::Up,
                        _ => Button::Up,
                    };
                    FRLGSafariEncounterState::EnteringEncounter
                } else {
                    control.press_delay(self.next_dir.clone(), Delay::Twentieth);
                    self.next_dir = match self.next_dir {
                        Button::Up => Button::Down,
                        Button::Down => Button::Up,
                        Button::Left => Button::Right,
                        Button::Right => Button::Left,
                        _ => Button::Up,
                    };
                    let delay = 100;
                    self.create_wait_msecs(delay, FRLGSafariEncounterState::TryGetEncounter)
                }
            }
            FRLGSafariEncounterState::EnteringEncounter => {
                if in_encounter {
                    self.timer = SystemTime::now();
                    FRLGSafariEncounterState::WaitEncounterReady
                } else {
                    FRLGSafariEncounterState::EnteringEncounter
                }
            }
            FRLGSafariEncounterState::WaitEncounterReady => {
                if encounter_ready {
                    self.last_timer_duration = self.timer.elapsed().unwrap();
                    FRLGSafariEncounterState::PressA
                } else {
                    FRLGSafariEncounterState::WaitEncounterReady
                }
            }
            FRLGSafariEncounterState::PressA => {
                control.press(Button::A);
                self.create_wait_secs(2, FRLGSafariEncounterState::Detect)
            }
            FRLGSafariEncounterState::Detect => {
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
                    // log::info!("sprite = {}, duration = {}", detect.shiny, self.last_timer_duration > SHINY_DURATION);
                    log::info!("last duration = {:?}", self.last_timer_duration);
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
                                    game: Game::DiamondPearl,
                                    method: Method::RandomEncounter,
                                }),
                            });
                        }
                        FRLGSafariEncounterState::Done
                    } else {
                        FRLGSafariEncounterState::Run1Down
                    }
                } else {
                    log::error!("No detect result found");
                    FRLGSafariEncounterState::Done
                }
            }
            FRLGSafariEncounterState::Run1Down => {
                control.press(Button::Down);
                self.create_wait_msecs(200, FRLGSafariEncounterState::Run2Right)
            }
            FRLGSafariEncounterState::Run2Right => {
                control.press(Button::Right);
                self.create_wait_msecs(200, FRLGSafariEncounterState::Run3A)
            }
            FRLGSafariEncounterState::Run3A => {
                control.press(Button::A);
                self.create_wait_msecs(500, FRLGSafariEncounterState::Run4A)
            }
            FRLGSafariEncounterState::Run4A => {
                control.press(Button::A);
                let mut rng = rand::rng();
                let delay = 3000 + rng.random_range(0..2100);
                self.create_wait_msecs(delay, FRLGSafariEncounterState::TryGetEncounter)
            }
            FRLGSafariEncounterState::Done => FRLGSafariEncounterState::Done,
            FRLGSafariEncounterState::Wait(duration, next) => {
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
