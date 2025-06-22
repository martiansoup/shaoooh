use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime};

use crate::app::states::{Game, Method, RequestTransition, Transition, TransitionArg};
use crate::control::{Button, Delay, ShaooohControl};
use crate::hunt::{BaseHunt, HuntFSM, HuntResult};
use crate::vision::{Processing, ProcessingResult};

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum RSSafariEncounterState {
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
    Wait(Duration, Box<RSSafariEncounterState>),
}

const SHINY_DURATION: Duration = Duration::from_millis(3250);

pub(crate) struct RSSafariEncounter {
    pub(crate) base: BaseHunt,
    pub(crate) state: RSSafariEncounterState,
    pub(crate) next_dir: Button,
    pub(crate) timer: SystemTime,
    pub(crate) last_timer_duration: Duration,
    //pub(crate) stats_file: File,
}

impl RSSafariEncounter {
    fn create_wait_secs(
        &mut self,
        d: u64,
        state: RSSafariEncounterState,
    ) -> RSSafariEncounterState {
        self.base.wait_start = SystemTime::now();
        RSSafariEncounterState::Wait(Duration::from_secs(d), Box::new(state))
    }

    fn create_wait_msecs(
        &mut self,
        d: u64,
        state: RSSafariEncounterState,
    ) -> RSSafariEncounterState {
        self.base.wait_start = SystemTime::now();
        RSSafariEncounterState::Wait(Duration::from_millis(d), Box::new(state))
    }
}

impl HuntFSM for RSSafariEncounter {
    fn processing(&self) -> Vec<Processing> {
        if self.state == RSSafariEncounterState::Detect {
            // TODO hardcoded list for safari zone 1
            vec![Processing::Sprite(
                Game::RubySapphire,
                vec![25, 43, 44, 84, 177, 203, 202],
                false,
            )]
        } else if self.state == RSSafariEncounterState::TryGetEncounter {
            vec![Processing::FRLG_START_ENCOUNTER]
        } else if self.state == RSSafariEncounterState::EnteringEncounter {
            vec![Processing::FRLG_IN_ENCOUNTER]
        } else if self.state == RSSafariEncounterState::WaitEncounterReady {
            vec![
                Processing::FRLG_IN_ENCOUNTER,
                Processing::FRLG_ENCOUNTER_READY,
            ]
        } else {
            Vec::new()
        }
    }

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult {
        let incr_encounters = self.state == RSSafariEncounterState::Detect;
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
            RSSafariEncounterState::TryGetEncounter => {
                if enter_encounter {
                    self.next_dir = match self.next_dir {
                        Button::Up => Button::Left,
                        Button::Down => Button::Left,
                        Button::Left => Button::Up,
                        Button::Right => Button::Up,
                        _ => Button::Up,
                    };
                    RSSafariEncounterState::EnteringEncounter
                } else {
                    control.press_delay(self.next_dir.clone(), Delay::Twentieth);
                    self.next_dir = match self.next_dir {
                        Button::Up => Button::Down,
                        Button::Down => Button::Up,
                        Button::Left => Button::Right,
                        Button::Right => Button::Left,
                        _ => Button::Up,
                    };
                    let delay = 200;
                    self.create_wait_msecs(delay, RSSafariEncounterState::TryGetEncounter)
                }
            }
            RSSafariEncounterState::EnteringEncounter => {
                if in_encounter {
                    self.timer = SystemTime::now();
                    RSSafariEncounterState::WaitEncounterReady
                } else {
                    RSSafariEncounterState::EnteringEncounter
                }
            }
            RSSafariEncounterState::WaitEncounterReady => {
                if encounter_ready {
                    self.last_timer_duration = self.timer.elapsed().unwrap();
                    RSSafariEncounterState::PressA
                } else {
                    RSSafariEncounterState::WaitEncounterReady
                }
            }
            RSSafariEncounterState::PressA => {
                control.press(Button::A);
                self.create_wait_secs(2, RSSafariEncounterState::Detect)
            }
            RSSafariEncounterState::Detect => {
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
                        RSSafariEncounterState::Done
                    } else {
                        RSSafariEncounterState::Run1Down
                    }
                } else {
                    log::error!("No detect result found");
                    RSSafariEncounterState::Done
                }
            }
            RSSafariEncounterState::Run1Down => {
                control.press(Button::Down);
                self.create_wait_secs(1, RSSafariEncounterState::Run2Right)
            }
            RSSafariEncounterState::Run2Right => {
                control.press(Button::Right);
                self.create_wait_secs(1, RSSafariEncounterState::Run3A)
            }
            RSSafariEncounterState::Run3A => {
                control.press(Button::A);
                self.create_wait_secs(1, RSSafariEncounterState::Run4A)
            }
            RSSafariEncounterState::Run4A => {
                control.press(Button::A);
                self.create_wait_secs(3, RSSafariEncounterState::TryGetEncounter)
            }
            RSSafariEncounterState::Done => RSSafariEncounterState::Done,
            RSSafariEncounterState::Wait(duration, next) => {
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
