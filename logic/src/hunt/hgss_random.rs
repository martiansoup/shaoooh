use std::time::{Duration, SystemTime};

use crate::app::states::{Game, Method, RequestTransition, Transition, TransitionArg};
use crate::control::{Button, Delay, ShaooohControl};
use crate::hunt::{BaseHunt, HuntFSM, HuntResult};
use crate::vision::{Processing, ProcessingResult};

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum HGSSRandomEncounterState {
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
    Wait(Duration, Box<HGSSRandomEncounterState>),
}

const SHINY_DURATION: Duration = Duration::from_millis(7075);

pub(crate) struct HGSSRandomEncounter {
    pub(crate) base: BaseHunt,
    pub(crate) state: HGSSRandomEncounterState,
    pub(crate) next_dir: Button,
    pub(crate) timer: SystemTime,
    pub(crate) last_timer_duration: Duration,
    pub(crate) min_shiny: Duration,
    pub(crate) max_shiny: Duration,
    pub(crate) min_normal: Duration,
    pub(crate) max_normal: Duration,
}

impl HGSSRandomEncounter {
    fn create_wait_secs(
        &mut self,
        d: u64,
        state: HGSSRandomEncounterState,
    ) -> HGSSRandomEncounterState {
        self.base.wait_start = SystemTime::now();
        HGSSRandomEncounterState::Wait(Duration::from_secs(d), Box::new(state))
    }

    fn create_wait_msecs(
        &mut self,
        d: u64,
        state: HGSSRandomEncounterState,
    ) -> HGSSRandomEncounterState {
        self.base.wait_start = SystemTime::now();
        HGSSRandomEncounterState::Wait(Duration::from_millis(d), Box::new(state))
    }
}

impl HuntFSM for HGSSRandomEncounter {
    fn processing(&self) -> Vec<Processing> {
        if self.state == HGSSRandomEncounterState::Detect {
            // TODO hardcoded list for route 202
            vec![Processing::Sprite(
                Game::HeartGoldSoulSilver,
                vec![132, 83, 241, 20, 21, 44, 22],
                false,
            )]
        } else if self.state == HGSSRandomEncounterState::TryGetEncounter {
            vec![Processing::DP_START_ENCOUNTER]
        } else if self.state == HGSSRandomEncounterState::EnteringEncounter {
            vec![Processing::DP_IN_ENCOUNTER]
        } else if self.state == HGSSRandomEncounterState::WaitEncounterReady {
            vec![
                Processing::DP_IN_ENCOUNTER,
                Processing::DP_SAFARI_ENCOUNTER_READY,
            ]
        } else if self.state == HGSSRandomEncounterState::LeavingEncounter {
            vec![Processing::DP_IN_ENCOUNTER, Processing::DP_START_ENCOUNTER]
        } else {
            Vec::new()
        }
    }

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult {
        let incr_encounters = self.state == HGSSRandomEncounterState::Detect;
        let mut transition = None;
        let mut detect_result = None;
        let mut enter_encounter = false;
        let mut in_encounter = false;
        let mut encounter_ready = false;

        for r in results {
            match r.process {
                Processing::Sprite(_, _, _) => detect_result = Some(r),
                Processing::DP_START_ENCOUNTER => enter_encounter = r.met,
                Processing::DP_IN_ENCOUNTER => in_encounter = r.met,
                Processing::DP_SAFARI_ENCOUNTER_READY => encounter_ready = r.met,
                _ => {}
            }
        }

        let old_state = self.state.clone();

        self.state = match &self.state {
            HGSSRandomEncounterState::TryGetEncounter => {
                if enter_encounter {
                    self.next_dir = match self.next_dir {
                        Button::Up => Button::Right,
                        Button::Down => Button::Right,
                        Button::Left => Button::Up,
                        Button::Right => Button::Up,
                        _ => Button::Up,
                    };
                    HGSSRandomEncounterState::EnteringEncounter
                } else {
                    control.press_delay(self.next_dir.clone(), Delay::Twentieth);
                    self.next_dir = match self.next_dir {
                        Button::Up => Button::Down,
                        Button::Left => Button::Right,
                        Button::Down => Button::Up,
                        Button::Right => Button::Left,
                        _ => Button::Up,
                    };
                    control.press(Button::B);
                    self.create_wait_msecs(200, HGSSRandomEncounterState::TryGetEncounter)
                }
            }
            HGSSRandomEncounterState::EnteringEncounter => {
                if in_encounter {
                    self.timer = SystemTime::now();
                    HGSSRandomEncounterState::WaitEncounterReady
                } else {
                    HGSSRandomEncounterState::EnteringEncounter
                }
            }
            HGSSRandomEncounterState::WaitEncounterReady => {
                if encounter_ready {
                    self.last_timer_duration = self.timer.elapsed().unwrap();
                    HGSSRandomEncounterState::Detect
                } else {
                    HGSSRandomEncounterState::WaitEncounterReady
                }
            }
            HGSSRandomEncounterState::Detect => {
                // TODO change to trace or remove?
                log::info!(
                    "Durations: min_normal={:?},max_normal={:?} - min_shiny={:?},max_shiny={:?} - last = {:?}",
                    self.min_normal,
                    self.max_normal,
                    self.min_shiny,
                    self.max_shiny,
                    self.last_timer_duration
                );
                if let Some(detect) = detect_result {
                    log::info!(
                        "Shiny: sprite = {}, duration = {}",
                        detect.shiny,
                        (self.last_timer_duration > SHINY_DURATION)
                    );
                    //if detect.shiny || (self.last_timer_duration > SHINY_DURATION) {
                    if self.last_timer_duration > SHINY_DURATION {
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
                                    game: Game::HeartGoldSoulSilver,
                                    method: Method::RandomEncounter,
                                }),
                            });
                        }
                        HGSSRandomEncounterState::Done
                    } else {
                        if self.last_timer_duration > self.max_normal {
                            self.max_normal = self.last_timer_duration;
                        }
                        if self.last_timer_duration < self.min_normal {
                            self.min_normal = self.last_timer_duration;
                        }
                        HGSSRandomEncounterState::WaitIntimidate
                    }
                } else {
                    log::error!("No detect result found");
                    HGSSRandomEncounterState::Done
                }
            }
            HGSSRandomEncounterState::WaitIntimidate => {
                self.create_wait_secs(2, HGSSRandomEncounterState::Run1Down)
            }
            HGSSRandomEncounterState::Run1Down => {
                control.press(Button::Down);
                self.create_wait_secs(1, HGSSRandomEncounterState::Run2Down)
            }
            HGSSRandomEncounterState::Run2Down => {
                control.press(Button::Down);
                self.create_wait_secs(1, HGSSRandomEncounterState::Run3Right)
            }
            HGSSRandomEncounterState::Run3Right => {
                control.press(Button::Right);
                self.create_wait_secs(1, HGSSRandomEncounterState::Run4A)
            }
            HGSSRandomEncounterState::Run4A => {
                control.press(Button::A);
                self.create_wait_secs(5, HGSSRandomEncounterState::LeavingEncounter)
            }
            HGSSRandomEncounterState::LeavingEncounter => {
                if !enter_encounter && !in_encounter {
                    HGSSRandomEncounterState::TryGetEncounter
                } else {
                    HGSSRandomEncounterState::LeavingEncounter
                }
            }
            HGSSRandomEncounterState::Done => HGSSRandomEncounterState::Done,
            HGSSRandomEncounterState::Wait(duration, next) => {
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
