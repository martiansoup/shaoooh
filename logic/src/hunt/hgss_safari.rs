use std::time::{Duration, SystemTime};

use crate::app::states::{Game, Method, RequestTransition, Transition, TransitionArg};
use crate::control::{Button, ShaooohControl};
use crate::hunt::{BaseHunt, HuntFSM, HuntResult};
use crate::vision::{Processing, ProcessingResult};

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum HGSSSafariEncounterState {
    Start,
    Sweet1PressX,
    Sweet2PressA,
    Sweet3PressA,
    Sweet4PressLeft,
    Sweet5PressA,
    WaitEnter,
    EnteringEncounter,
    WaitEncounterReady,
    Detect,
    Run1Down,
    Run2Down,
    Run3Right,
    Run4A,
    Done,
    LeavingEncounter,
    Wait(Duration, Box<HGSSSafariEncounterState>),
}

const SHINY_DURATION: Duration = Duration::from_millis(4650);

pub(crate) struct HGSSSafariEncounter {
    pub(crate) base: BaseHunt,
    pub(crate) state: HGSSSafariEncounterState,
    pub(crate) timer: SystemTime,
    pub(crate) last_timer_duration: Duration,
    pub(crate) min_shiny: Duration,
    pub(crate) max_shiny: Duration,
    pub(crate) min_normal: Duration,
    pub(crate) max_normal: Duration,
}

impl HGSSSafariEncounter {
    fn create_wait_secs(
        &mut self,
        d: u64,
        state: HGSSSafariEncounterState,
    ) -> HGSSSafariEncounterState {
        self.base.wait_start = SystemTime::now();
        HGSSSafariEncounterState::Wait(Duration::from_secs(d), Box::new(state))
    }

    fn create_wait_msecs(
        &mut self,
        d: u64,
        state: HGSSSafariEncounterState,
    ) -> HGSSSafariEncounterState {
        self.base.wait_start = SystemTime::now();
        HGSSSafariEncounterState::Wait(Duration::from_millis(d), Box::new(state))
    }
}

impl HuntFSM for HGSSSafariEncounter {
    fn processing(&self) -> Vec<Processing> {
        if self.state == HGSSSafariEncounterState::Detect {
            // TODO hardcoded list for safari mountain
            vec![Processing::Sprite(
                Game::HeartGoldSoulSilver,
                vec![19, 20, 108, 82, 246, 41, 42],
                false,
            )]
        } else if self.state == HGSSSafariEncounterState::WaitEnter {
            vec![Processing::DP_START_ENCOUNTER]
        } else if self.state == HGSSSafariEncounterState::EnteringEncounter {
            vec![Processing::DP_IN_ENCOUNTER]
        } else if self.state == HGSSSafariEncounterState::WaitEncounterReady {
            vec![
                Processing::DP_IN_ENCOUNTER,
                Processing::DP_SAFARI_ENCOUNTER_READY,
            ]
        } else if self.state == HGSSSafariEncounterState::LeavingEncounter {
            vec![Processing::DP_IN_ENCOUNTER, Processing::DP_START_ENCOUNTER]
        } else {
            Vec::new()
        }
    }

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult {
        let incr_encounters = self.state == HGSSSafariEncounterState::Detect;
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
            HGSSSafariEncounterState::Start => HGSSSafariEncounterState::Sweet1PressX,
            HGSSSafariEncounterState::Sweet1PressX => {
                // Open menu
                control.press(Button::X);
                self.create_wait_msecs(500, HGSSSafariEncounterState::Sweet2PressA)
            }
            HGSSSafariEncounterState::Sweet2PressA => {
                // Open pkmn
                control.press(Button::A);
                self.create_wait_msecs(1500, HGSSSafariEncounterState::Sweet3PressA)
            }
            HGSSSafariEncounterState::Sweet3PressA => {
                // Sel mon
                control.press(Button::A);
                self.create_wait_msecs(500, HGSSSafariEncounterState::Sweet4PressLeft)
            }
            HGSSSafariEncounterState::Sweet4PressLeft => {
                // Sel sweet scene
                control.press(Button::Left);
                self.create_wait_msecs(1000, HGSSSafariEncounterState::Sweet5PressA)
            }
            HGSSSafariEncounterState::Sweet5PressA => {
                // Press it
                control.press(Button::A);
                self.create_wait_msecs(2000, HGSSSafariEncounterState::WaitEnter)
            }
            HGSSSafariEncounterState::WaitEnter => {
                if enter_encounter {
                    HGSSSafariEncounterState::EnteringEncounter
                } else {
                    HGSSSafariEncounterState::WaitEnter
                }
            }
            HGSSSafariEncounterState::EnteringEncounter => {
                if in_encounter {
                    self.timer = SystemTime::now();
                    HGSSSafariEncounterState::WaitEncounterReady
                } else {
                    HGSSSafariEncounterState::EnteringEncounter
                }
            }
            HGSSSafariEncounterState::WaitEncounterReady => {
                if encounter_ready {
                    self.last_timer_duration = self.timer.elapsed().unwrap();
                    HGSSSafariEncounterState::Detect
                } else {
                    HGSSSafariEncounterState::WaitEncounterReady
                }
            }
            HGSSSafariEncounterState::Detect => {
                // TODO change to trace or remove?
                log::info!(
                    "Durations: min_normal={:?},max_normal={:?} - min_shiny={:?},max_shiny={:?}",
                    self.min_normal,
                    self.max_normal,
                    self.min_shiny,
                    self.max_shiny
                );
                if let Some(detect) = detect_result {
                    log::info!("duration = {:?}", self.last_timer_duration);
                    log::info!(
                        "Shiny: sprite = {}, duration = {}",
                        detect.shiny,
                        (self.last_timer_duration > SHINY_DURATION)
                    );
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
                                    game: Game::HeartGoldSoulSilver,
                                    method: Method::SafariZone,
                                }),
                            });
                        }
                        HGSSSafariEncounterState::Done
                    } else {
                        if self.last_timer_duration > self.max_normal {
                            self.max_normal = self.last_timer_duration;
                        }
                        if self.last_timer_duration < self.min_normal {
                            self.min_normal = self.last_timer_duration;
                        }
                        self.create_wait_msecs(500, HGSSSafariEncounterState::Run1Down)
                    }
                } else {
                    log::error!("No detect result found");
                    HGSSSafariEncounterState::Done
                }
            }
            HGSSSafariEncounterState::Run1Down => {
                control.press(Button::Down);
                self.create_wait_secs(1, HGSSSafariEncounterState::Run2Down)
            }
            HGSSSafariEncounterState::Run2Down => {
                control.press(Button::Down);
                self.create_wait_secs(1, HGSSSafariEncounterState::Run3Right)
            }
            HGSSSafariEncounterState::Run3Right => {
                control.press(Button::Right);
                self.create_wait_secs(1, HGSSSafariEncounterState::Run4A)
            }
            HGSSSafariEncounterState::Run4A => {
                control.press(Button::A);
                self.create_wait_secs(4, HGSSSafariEncounterState::LeavingEncounter)
            }
            HGSSSafariEncounterState::LeavingEncounter => {
                if !enter_encounter && !in_encounter {
                    self.create_wait_secs(2, HGSSSafariEncounterState::Start)
                } else {
                    HGSSSafariEncounterState::LeavingEncounter
                }
            }
            HGSSSafariEncounterState::Done => HGSSSafariEncounterState::Done,
            HGSSSafariEncounterState::Wait(duration, next) => {
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
