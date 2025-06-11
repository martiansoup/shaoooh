use std::time::{Duration, SystemTime};

use crate::app::states::{Game, Method, RequestTransition, Transition, TransitionArg};
use crate::control::{Button, ShaooohControl};
use crate::hunt::{BaseHunt, HuntFSM, HuntResult};
use crate::vision::{Processing, ProcessingResult};

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum DPRandomEncounterState {
    TryGetEncounter,
    EnteringEncounter,
    WaitEncounterReady,
    Detect,
    WaitIntimidate, // TODO check for icon rather than delay
    Run1Down,
    Run2Down,
    Run3Right,
    Run4A,
    Done,
    LeavingEncounter,
    Wait(Duration, Box<DPRandomEncounterState>),
}

const SHINY_DURATION: Duration = Duration::from_millis(10200);

pub(crate) struct DPRandomEncounter {
    pub(crate) base: BaseHunt,
    pub(crate) state: DPRandomEncounterState,
    pub(crate) next_dir: Button,
    pub(crate) timer: SystemTime,
    pub(crate) last_timer_duration: Duration,
    pub(crate) min_shiny: Duration,
    pub(crate) max_shiny: Duration,
    pub(crate) min_normal: Duration,
    pub(crate) max_normal: Duration,
}

impl DPRandomEncounter {
    fn create_wait_secs(
        &mut self,
        d: u64,
        state: DPRandomEncounterState,
    ) -> DPRandomEncounterState {
        self.base.wait_start = SystemTime::now();
        DPRandomEncounterState::Wait(Duration::from_secs(d), Box::new(state))
    }

    fn create_wait_msecs(
        &mut self,
        d: u64,
        state: DPRandomEncounterState,
    ) -> DPRandomEncounterState {
        self.base.wait_start = SystemTime::now();
        DPRandomEncounterState::Wait(Duration::from_millis(d), Box::new(state))
    }
}

impl HuntFSM for DPRandomEncounter {
    fn processing(&self) -> Vec<Processing> {
        if self.state == DPRandomEncounterState::Detect {
            // TODO hardcoded list for route 202
            vec![Processing::Sprite(
                Game::DiamondPearl,
                vec![396, 399, 401, 403],
                false,
            )]
        } else if self.state == DPRandomEncounterState::TryGetEncounter {
            vec![Processing::DPStartEncounter]
        } else if self.state == DPRandomEncounterState::EnteringEncounter {
            vec![Processing::DPInEncounter]
        } else if self.state == DPRandomEncounterState::WaitEncounterReady {
            vec![Processing::DPInEncounter, Processing::DPEncounterReady]
        } else if self.state == DPRandomEncounterState::LeavingEncounter {
            vec![Processing::DPInEncounter, Processing::DPStartEncounter]
        } else {
            Vec::new()
        }
    }

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult {
        let incr_encounters = self.state == DPRandomEncounterState::Detect;
        let mut transition = None;
        let mut detect_result = None;
        let mut enter_encounter = false;
        let mut in_encounter = false;
        let mut encounter_ready = false;

        for r in results {
            match r.process {
                Processing::Sprite(_, _, _) => detect_result = Some(r),
                Processing::DPStartEncounter => enter_encounter = r.met,
                Processing::DPInEncounter => in_encounter = r.met,
                Processing::DPEncounterReady => encounter_ready = r.met,
                _ => {}
            }
        }

        let old_state = self.state.clone();

        self.state = match &self.state {
            DPRandomEncounterState::TryGetEncounter => {
                if enter_encounter {
                    DPRandomEncounterState::EnteringEncounter
                } else {
                    control.press(self.next_dir.clone());
                    self.next_dir = match self.next_dir {
                        Button::Up => Button::Left,
                        Button::Left => Button::Down,
                        Button::Down => Button::Right,
                        Button::Right => Button::Up,
                        _ => Button::Up,
                    };
                    self.create_wait_msecs(200, DPRandomEncounterState::TryGetEncounter)
                }
            }
            DPRandomEncounterState::EnteringEncounter => {
                if in_encounter {
                    self.timer = SystemTime::now();
                    DPRandomEncounterState::WaitEncounterReady
                } else {
                    DPRandomEncounterState::EnteringEncounter
                }
            }
            DPRandomEncounterState::WaitEncounterReady => {
                if encounter_ready {
                    self.last_timer_duration = self.timer.elapsed().unwrap();
                    DPRandomEncounterState::Detect
                } else {
                    DPRandomEncounterState::WaitEncounterReady
                }
            }
            DPRandomEncounterState::Detect => {
                // TODO change to trace or remove?
                log::info!(
                    "Durations: min_normal={:?},max_normal={:?} - min_shiny={:?},max_shiny={:?}",
                    self.min_normal,
                    self.max_normal,
                    self.min_shiny,
                    self.max_shiny
                );
                if let Some(detect) = detect_result {
                    if detect.shiny || (self.last_timer_duration > SHINY_DURATION) {
                        if self.last_timer_duration > self.max_shiny {
                            self.max_shiny = self.last_timer_duration;
                        }
                        if self.last_timer_duration < self.min_shiny {
                            self.min_shiny = self.last_timer_duration;
                        }
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
                        DPRandomEncounterState::Done
                    } else {
                        if self.last_timer_duration > self.max_normal {
                            self.max_normal = self.last_timer_duration;
                        }
                        if self.last_timer_duration < self.min_normal {
                            self.min_normal = self.last_timer_duration;
                        }
                        DPRandomEncounterState::WaitIntimidate
                    }
                } else {
                    log::error!("No detect result found");
                    DPRandomEncounterState::Done
                }
            }
            DPRandomEncounterState::WaitIntimidate => {
                self.create_wait_secs(7, DPRandomEncounterState::Run1Down)
            }
            DPRandomEncounterState::Run1Down => {
                control.press(Button::Down);
                self.create_wait_secs(1, DPRandomEncounterState::Run2Down)
            }
            DPRandomEncounterState::Run2Down => {
                control.press(Button::Down);
                self.create_wait_secs(1, DPRandomEncounterState::Run3Right)
            }
            DPRandomEncounterState::Run3Right => {
                control.press(Button::Right);
                self.create_wait_secs(1, DPRandomEncounterState::Run4A)
            }
            DPRandomEncounterState::Run4A => {
                control.press(Button::A);
                self.create_wait_secs(5, DPRandomEncounterState::LeavingEncounter)
            }
            DPRandomEncounterState::LeavingEncounter => {
                if !enter_encounter && !in_encounter {
                    DPRandomEncounterState::TryGetEncounter
                } else {
                    DPRandomEncounterState::LeavingEncounter
                }
            }
            DPRandomEncounterState::Done => DPRandomEncounterState::Done,
            DPRandomEncounterState::Wait(duration, next) => {
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
