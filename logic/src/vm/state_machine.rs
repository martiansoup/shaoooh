use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime};

use crate::vm::adapter::StateParser;
use crate::vm::state::{GroupOp, InputOp, InternalOp, State};

use crate::fsm::{StateId, StateMachine};
use crate::hunt::HuntResult;
use crate::hunt::InternalHuntState;

use crate::app::{Game, Method, RequestTransition, Transition};

use crate::vision::{Processing, ProcessingResult};

#[derive(Debug)]
pub struct ParsedStateMachine {
    states: Vec<State>,
}

#[derive(Debug)]
pub enum ParsedStateMachineError {
    BranchNotFound,
    DuplicateLabel,
    FailedToParseProcessing(String),
    FailedToParseOutputs(String),
}

type CheckType =
    Box<dyn Fn(&Vec<ProcessingResult>, &mut InternalHuntState) -> Option<(usize, HuntResult)>>;

impl ParsedStateMachine {
    pub fn new(states: Vec<State>) -> Self {
        Self { states }
    }

    // Hunt is: StateMachine<Processing, ProcessingResult, HuntStateOutput, HuntResult, InternalHuntState>,
    // pub struct StateMachine<InputKind, InputValue, StateOutput, StateTransition, InternalState>
    pub fn build<T>(
        self,
        target: u32,
        game: Game,
        _method: Method,
        strict: bool,
    ) -> Result<
        StateMachine<Processing, ProcessingResult, T, HuntResult, InternalHuntState>,
        ParsedStateMachineError,
    >
    where
        T: StateParser + std::fmt::Debug + Clone,
    {
        let b = AtomicBool::new(true);
        let atomic = Arc::new(b);
        let mut fsm = StateMachine::new(InternalHuntState::new(atomic));

        let mut mapping = HashMap::new();

        for (i, s) in self.states.iter().enumerate() {
            if let Some(tag) = s.tag() {
                if mapping.contains_key(tag) {
                    return Err(ParsedStateMachineError::DuplicateLabel);
                }
                mapping.insert(tag.to_string(), i);
            }
        }

        let num_states = self.states.len();

        for (i, s) in self.states.into_iter().enumerate() {
            let name = match s.tag() {
                Some(s) => s.to_string(),
                None => format!("state#{}", i),
            };
            let id = StateId::new(i, name.clone(), name);

            let outputs: Vec<Result<T, String>> = s
                .outputs()
                .iter()
                .map(|x| T::parse(x, game.clone(), true))
                .collect();
            let any_output_error = outputs
                .iter()
                .filter(|x| x.is_err())
                .map(|x| x.clone().unwrap_err())
                .reduce(|acc, e| acc + " " + &e);

            if let Some(err) = any_output_error {
                return Err(ParsedStateMachineError::FailedToParseOutputs(err));
            }

            let outputs: Vec<T> = outputs.into_iter().flatten().collect();

            let delay = s.delay();

            let mut p_inputs: Vec<Result<Processing, String>> = s
                .inputs_grp1()
                .iter()
                .map(|x| Processing::parse(x, game.clone(), strict))
                .collect();
            p_inputs.extend(
                s.inputs_grp2()
                    .iter()
                    .map(|x| Processing::parse(x, game.clone(), strict)),
            );

            let any_proc_error = p_inputs
                .iter()
                .filter(|x| x.is_err())
                .map(|x| x.clone().unwrap_err())
                .reduce(|acc, e| acc + " " + &e);

            if let Some(err) = any_proc_error {
                return Err(ParsedStateMachineError::FailedToParseProcessing(err));
            }

            let inputs: Vec<Processing> = p_inputs.into_iter().flatten().collect();

            let mut next_states = vec![];
            let next_wrapped = if (i + 1) == num_states { 0 } else { i + 1 };

            let mut has_positive = false;
            let mut positive = i;
            let mut negative = i;

            if let Some(state) = s.positive() {
                if let Some(v) = mapping.get(state) {
                    next_states.push(*v);
                    positive = *v;
                    has_positive = true;
                } else {
                    return Err(ParsedStateMachineError::BranchNotFound);
                }
            }

            if let Some(state) = s.negative() {
                if let Some(v) = mapping.get(state) {
                    next_states.push(*v);
                    negative = *v;
                } else {
                    return Err(ParsedStateMachineError::BranchNotFound);
                }
            }

            // If deadend, same state can be next
            if s.is_deadend() {
                next_states.push(i);
            }

            // If no processing, add next state
            if !(s.any_processing() || s.any_branch()) {
                next_states.push(next_wrapped);
            }

            // If any processing and a branch is not specified, holds in same
            // state until met
            if (s.any_processing() || s.any_branch())
                && (s.positive().is_none() || s.negative().is_none())
            {
                next_states.push(i);
            }

            let check: CheckType = if s.simple() {
                Box::new(
                    move |_x: &Vec<ProcessingResult>, _int: &mut InternalHuntState| {
                        Some((next_wrapped, HuntResult::default()))
                    },
                )
            } else {
                let deadend = s.is_deadend();
                let any_proc_mod = s.any_proc_mod();
                let any_proc = s.any_processing();
                let any_branch = s.any_branch();
                let inputs_grp1: Vec<Processing> = s
                    .inputs_grp1()
                    .iter()
                    .flat_map(|x| Processing::parse(x, game.clone(), strict))
                    .collect();
                let inputs_grp2: Vec<Processing> = s
                    .inputs_grp1()
                    .iter()
                    .flat_map(|x| Processing::parse(x, game.clone(), strict))
                    .collect();
                let modifiers = s.modifiers().to_vec();
                let grp1_op = s.grp1_op();
                let grp2_op = s.grp2_op();
                let grp1_2_op = s.grp1_2_op();

                let proc_mod = s.get_proc_mod();
                let branch = s.get_branch();

                Box::new(
                    move |x: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                        let mut hunt_res = HuntResult::default();

                        for m in &modifiers {
                            match m {
                                // Modify internal state
                                InternalOp::SetCounter(u) => int.counter = *u,
                                InternalOp::StartTimer => int.time = SystemTime::now(),
                                InternalOp::UpdateTimer => {
                                    int.last_duration =
                                        int.time.elapsed().expect("Failed to get time")
                                }
                                InternalOp::DecrCounter => int.counter -= 1,
                                InternalOp::IncrCounter => int.counter += 1,
                                InternalOp::Toggle => int.toggle = !int.toggle,
                                InternalOp::SetAtomic => int.atomic.store(true, Ordering::Release),
                                InternalOp::ClearAtomic => {
                                    int.atomic.store(false, Ordering::Release)
                                }
                                // Branch type states
                                InternalOp::LastDelay(..) => {}
                                InternalOp::CounterZero => {}
                                InternalOp::CounterValue(..) => {}
                                InternalOp::CheckToggle => {}
                                InternalOp::Deadend => {}
                                // Processing modifiers
                                InternalOp::ProcDelay(..) => {}
                                InternalOp::ProcDelayRange(..) => {}
                                // Modfiying result
                                InternalOp::FoundTarget => {
                                    hunt_res.transition = Some(RequestTransition {
                                        transition: Transition::FoundTarget,
                                        arg: None,
                                    })
                                }
                                InternalOp::IncrEncounters => hunt_res.incr_encounters = true,
                            }
                        }

                        if deadend {
                            // Branch to self
                            Some((i, hunt_res))
                        } else if any_proc {
                            let mut grp1_results =
                                x.iter().filter(|s| inputs_grp1.contains(&s.process));
                            let mut grp2_results =
                                x.iter().filter(|s| inputs_grp2.contains(&s.process));

                            let grp1_res = match grp1_op {
                                InputOp::And => grp1_results.all(|b| b.met),
                                InputOp::Or => grp1_results.any(|b| b.met),
                            };

                            let grp2_res = match grp2_op {
                                InputOp::And => grp2_results.all(|b| b.met),
                                InputOp::Or => grp2_results.any(|b| b.met),
                            };

                            let mut r = if inputs_grp2.is_empty() {
                                grp1_res
                            } else {
                                match grp1_2_op {
                                    GroupOp::And => grp1_res && grp2_res,
                                    GroupOp::AndNot => grp1_res && !grp2_res,
                                }
                            };

                            if any_proc_mod {
                                let met = match &proc_mod {
                                    Some(InternalOp::ProcDelay(u)) => {
                                        log::debug!(
                                            "Processing modifier, duration = ({:?}/{:?})",
                                            int.last_duration,
                                            u
                                        );
                                        int.last_duration > Duration::from_millis(*u)
                                    }
                                    Some(InternalOp::ProcDelayRange(u, r)) => {
                                        log::debug!(
                                            "Processing modifier, duration = ({:?}/{:?}) or {:?}",
                                            int.last_duration,
                                            u,
                                            r
                                        );
                                        let r_dur = Duration::from_millis(r.start)
                                            ..Duration::from_millis(r.end);
                                        int.last_duration > Duration::from_millis(*u)
                                            || r_dur.contains(&int.last_duration)
                                    }
                                    _ => {
                                        panic!("Unexpected processing modifier");
                                    }
                                };
                                // If a delay is met, force condition met
                                r = r || met;
                                log::info!("Processing modifier = {}", met);
                            }

                            // Shiny in result overrides result to true
                            if x.iter().any(|s| s.shiny) {
                                r = true;

                                let transition =
                                    if x.iter().any(|s| s.shiny && (target == s.species)) {
                                        Transition::FoundTarget
                                    } else {
                                        todo!("Need arg for non target");
                                        Transition::FoundNonTarget
                                    };

                                hunt_res.transition = Some(RequestTransition {
                                    transition,
                                    arg: None,
                                });

                                log::info!("Processing result indicated shiny",);
                            }

                            // Calc processing result and branch or None
                            if r {
                                Some((positive, hunt_res))
                            } else {
                                Some((negative, hunt_res))
                            }
                        } else if any_branch {
                            let branch_met = match branch {
                                Some(InternalOp::LastDelay(u)) => {
                                    int.last_duration > Duration::from_millis(u)
                                }
                                Some(InternalOp::CounterZero) => int.counter == 0,
                                Some(InternalOp::CounterValue(v)) => int.counter == v,
                                Some(InternalOp::CheckToggle) => int.toggle,
                                // Deadend handled separately, shouldn't reach here
                                // so included in catchall
                                _ => {
                                    panic!("Unexpected branch")
                                }
                            };

                            if branch_met {
                                Some((positive, hunt_res))
                            } else {
                                Some((negative, hunt_res))
                            }
                        } else {
                            // No processing, if a positive branch, take it else to next
                            if has_positive {
                                Some((positive, hunt_res))
                            } else {
                                Some((next_wrapped, hunt_res))
                            }
                        }
                    },
                )
            };

            fsm.add_state(id, outputs, delay, inputs, next_states, check);
        }

        Ok(fsm)
    }
}
