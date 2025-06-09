use crate::app::states::{Game, RequestTransition, Transition};
use crate::hunt::{BaseHunt, HuntFSM, HuntResult};
use crate::control::{Button, ShaooohControl};
use crate::vision::{Processing, ProcessingResult};

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum DPRandomEncounterState {
    TryGetEncounter,
    EnteringEncounter,
    WaitEncounterReady,
    Detect,
    Run1Down,
    Run2Down,
    Run3Right,
    Run4A,
    Done,
    LeavingEncounter
}

pub(crate) struct DPRandomEncounter {
    pub(crate) base: BaseHunt,
    pub(crate) state: DPRandomEncounterState,
    pub(crate) next_dir: Button
}

impl HuntFSM for DPRandomEncounter {
    fn processing(&self) -> Vec<Processing> {
        if self.state == DPRandomEncounterState::Detect {
            // TODO hardcoded list for route 202
            vec![Processing::Sprite(Game::DiamondPearl, vec![396, 399, 401, 403], false)]
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
                Processing::Sprite(_, _, _) => { detect_result = Some(r) },
                Processing::DPStartEncounter => { enter_encounter = r.met },
                Processing::DPInEncounter => { in_encounter = r.met },
                Processing::DPEncounterReady => {encounter_ready = r.met }
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
                        Button::Up => Button::Right,
                        Button::Right => Button::Down,
                        Button::Down => Button::Left,
                        Button::Left => Button::Up,
                        _ => Button::Up
                    };
                    DPRandomEncounterState::TryGetEncounter
                }
            }
            DPRandomEncounterState::EnteringEncounter => {
                if in_encounter {
                    DPRandomEncounterState::WaitEncounterReady
                } else {
                    DPRandomEncounterState::EnteringEncounter
                }
            }
            DPRandomEncounterState::WaitEncounterReady => {
                if encounter_ready {
                    DPRandomEncounterState::Detect
                } else {
                    DPRandomEncounterState::WaitEncounterReady
                }
            }
            DPRandomEncounterState::Detect => {
                if let Some(detect) = detect_result {
                    if detect.shiny {
                        if detect.species == self.base.target {
                            transition = Some(RequestTransition { transition: Transition::FoundTarget, arg: None });
                        } else {
                            transition = Some(RequestTransition { transition: Transition::FoundNonTarget, arg: None });
                        }
                        DPRandomEncounterState::Done
                    } else {
                        DPRandomEncounterState::Run1Down
                    }
                } else {
                    log::error!("No detect result found");
                    DPRandomEncounterState::Done
                }
            }
            DPRandomEncounterState::Run1Down => {
                control.press(Button::Down);
                DPRandomEncounterState::Run2Down
            }
            DPRandomEncounterState::Run2Down => {
                control.press(Button::Down);
                DPRandomEncounterState::Run3Right
            }
            DPRandomEncounterState::Run3Right => {
                control.press(Button::Right);
                DPRandomEncounterState::Run4A
            }
            DPRandomEncounterState::Run4A => {
                control.press(Button::A);
                DPRandomEncounterState::LeavingEncounter
            }
            DPRandomEncounterState::LeavingEncounter => {
                if !enter_encounter && !in_encounter {
                    DPRandomEncounterState::TryGetEncounter
                } else {
                    DPRandomEncounterState::LeavingEncounter
                }
            }
            DPRandomEncounterState::Done => {
                DPRandomEncounterState::Done
            }
        };

        if old_state != self.state {
            log::debug!("STATE = {:?} -> {:?}", old_state, self.state);
        }


        HuntResult { transition, incr_encounters }
    }

    fn cleanup(&mut self) {
        log::info!("Closing FSM");
    }
}
