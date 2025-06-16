use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime};

use crate::app::states::{Game, Method, RequestTransition, Transition, TransitionArg};
use crate::control::{Button, ShaooohControl};
use crate::hunt::{BaseHunt, HuntFSM, HuntResult};
use crate::vision::{Processing, ProcessingResult};

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum FRLGRandomEncounterState {
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
    Wait(Duration, Box<FRLGRandomEncounterState>),
}

const SHINY_DURATION: Duration = Duration::from_millis(3250);

pub(crate) struct FRLGRandomEncounter {
    pub(crate) base: BaseHunt,
    pub(crate) state: FRLGRandomEncounterState,
    pub(crate) next_dir: Button,
    pub(crate) timer: SystemTime,
    pub(crate) last_timer_duration: Duration,
    pub(crate) stats_file: File,
}

impl FRLGRandomEncounter {
    fn create_wait_secs(
        &mut self,
        d: u64,
        state: FRLGRandomEncounterState,
    ) -> FRLGRandomEncounterState {
        self.base.wait_start = SystemTime::now();
        FRLGRandomEncounterState::Wait(Duration::from_secs(d), Box::new(state))
    }

    fn create_wait_msecs(
        &mut self,
        d: u64,
        state: FRLGRandomEncounterState,
    ) -> FRLGRandomEncounterState {
        self.base.wait_start = SystemTime::now();
        FRLGRandomEncounterState::Wait(Duration::from_millis(d), Box::new(state))
    }
}

impl HuntFSM for FRLGRandomEncounter {
    fn processing(&self) -> Vec<Processing> {
        if self.state == FRLGRandomEncounterState::Detect {
            // TODO hardcoded list for route 202
            vec![Processing::Sprite(
                Game::FireRedLeafGreen,
                vec![16, 19],
                false,
            )]
        } else if self.state == FRLGRandomEncounterState::TryGetEncounter {
            vec![Processing::FRLG_START_ENCOUNTER]
        } else if self.state == FRLGRandomEncounterState::EnteringEncounter {
            vec![Processing::FRLG_IN_ENCOUNTER]
        } else if self.state == FRLGRandomEncounterState::WaitEncounterReady {
            vec![
                Processing::FRLG_IN_ENCOUNTER,
                Processing::FRLG_ENCOUNTER_READY,
            ]
        } else {
            Vec::new()
        }
    }

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult {
        let incr_encounters = self.state == FRLGRandomEncounterState::Detect;
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
            FRLGRandomEncounterState::TryGetEncounter => {
                if enter_encounter {
                    FRLGRandomEncounterState::EnteringEncounter
                } else {
                    control.press(self.next_dir.clone());
                    self.next_dir = match self.next_dir {
                        Button::Up => Button::Left,
                        Button::Left => Button::Down,
                        Button::Down => Button::Right,
                        Button::Right => Button::Up,
                        _ => Button::Up,
                    };
                    self.create_wait_msecs(200, FRLGRandomEncounterState::TryGetEncounter)
                }
            }
            FRLGRandomEncounterState::EnteringEncounter => {
                if in_encounter {
                    self.timer = SystemTime::now();
                    FRLGRandomEncounterState::WaitEncounterReady
                } else {
                    FRLGRandomEncounterState::EnteringEncounter
                }
            }
            FRLGRandomEncounterState::WaitEncounterReady => {
                if encounter_ready {
                    self.last_timer_duration = self.timer.elapsed().unwrap();
                    FRLGRandomEncounterState::PressA
                } else {
                    FRLGRandomEncounterState::WaitEncounterReady
                }
            }
            FRLGRandomEncounterState::PressA => {
                control.press(Button::A);
                self.create_wait_secs(5, FRLGRandomEncounterState::Detect)
            }
            FRLGRandomEncounterState::Detect => {
                if let Some(detect) = detect_result {
                    self.stats_file.write_all(
                        format!(
                            "species={},time={},shiny={}\n",
                            detect.species,
                            self.last_timer_duration.as_millis(),
                            detect.shiny
                        )
                        .as_bytes(),
                    );
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
                        FRLGRandomEncounterState::Done
                    } else {
                        FRLGRandomEncounterState::Run1Down
                    }
                } else {
                    log::error!("No detect result found");
                    FRLGRandomEncounterState::Done
                }
            }
            FRLGRandomEncounterState::Run1Down => {
                control.press(Button::Down);
                self.create_wait_secs(1, FRLGRandomEncounterState::Run2Right)
            }
            FRLGRandomEncounterState::Run2Right => {
                control.press(Button::Right);
                self.create_wait_secs(1, FRLGRandomEncounterState::Run3A)
            }
            FRLGRandomEncounterState::Run3A => {
                control.press(Button::A);
                self.create_wait_secs(1, FRLGRandomEncounterState::Run4A)
            }
            FRLGRandomEncounterState::Run4A => {
                control.press(Button::A);
                self.create_wait_secs(3, FRLGRandomEncounterState::TryGetEncounter)
            }
            FRLGRandomEncounterState::Done => FRLGRandomEncounterState::Done,
            FRLGRandomEncounterState::Wait(duration, next) => {
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
