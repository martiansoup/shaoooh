use crate::app::{Game, Method, RequestTransition, Transition, TransitionArg};
use crate::context::species;
use crate::fsm::{BoxedStateCheck, StateMachine};
use crate::hunt::state_machine::HuntStateOutput;
use crate::hunt::{BaseHunt, HuntFSM, HuntResult, InternalHuntState};
use crate::vision::{Processing, ProcessingResult};
use std::cmp::Eq;
use std::collections::HashMap;
use std::convert::AsRef;
use std::hash::Hash;
use std::ops::Range;
use std::time::{Duration, SystemTime};

pub type BoxedProcessFn =
    Box<dyn Fn(&Vec<ProcessingResult>, &mut InternalHuntState) -> Option<HuntResult>>;

pub struct StateDescription<K> {
    tag: K,
    inputs: Vec<Processing>,
    buttons: Vec<HuntStateOutput>,
    delay_msecs: Range<u64>,
    check: HashMap<K, BoxedProcessFn>,
}

struct FragmentState {
    tag: usize,
    name: String,
    inputs: Vec<Processing>,
    outputs: Vec<HuntStateOutput>,
    delay_msecs: Range<u64>,
    checks: HashMap<usize, BoxedProcessFn>,
}

struct FSMFragment {
    states: Vec<FragmentState>,
}

pub struct HuntFSMBuilder {
    fragments: Vec<FSMFragment>,
    base: BaseHunt,
}

impl HuntFSMBuilder {
    pub fn new(base: BaseHunt) -> Self {
        let fragments = Vec::new();
        HuntFSMBuilder { fragments, base }
    }

    pub fn game(&self) -> &Game {
        &self.base.game
    }

    pub fn method(&self) -> &Method {
        &self.base.method
    }

    pub fn target(&self) -> u32 {
        self.base.target
    }

    pub fn add_states<K: Hash + Eq + Clone + AsRef<str>>(
        &mut self,
        states: Vec<StateDescription<K>>,
    ) {
        let mut state_mapping = HashMap::new();
        let mut fragment = Vec::new();

        for s in &states {
            let index = state_mapping.len();
            state_mapping.insert(s.tag.clone(), index);
        }

        for s in states {
            let mut new_checks = HashMap::new();
            for chk in s.check {
                new_checks.insert(
                    *state_mapping.get(&chk.0).expect("Failed to get tag"),
                    chk.1,
                );
            }

            let fragment_state = FragmentState {
                tag: *state_mapping.get(&s.tag).expect("Failed to get tag"),
                name: s.tag.as_ref().to_string(),
                inputs: s.inputs,
                outputs: s.buttons,
                delay_msecs: s.delay_msecs,
                checks: new_checks,
            };
            fragment.push(fragment_state);
        }

        self.fragments.push(FSMFragment { states: fragment });
    }

    pub fn build(self) -> HuntFSM {
        let mut last_index = 0;
        let mut fsm = StateMachine::new();

        let last_fragment = self.fragments.len() - 1;
        let mut findex = 0;
        for fragment in self.fragments {
            let fragment_first = last_index;
            let fragment_last = last_index + fragment.states.len() - 1;
            for state in fragment.states {
                let tag = state.tag + fragment_first;
                // If last state and last fragment, loop to start, else go to next state
                let next_state = if findex == last_fragment && tag == fragment_last {
                    0
                } else {
                    tag + 1
                };
                let name = state.name;
                let debug_name = format!(
                    "fragment{}-{} [{} next state(s)]",
                    findex,
                    name,
                    state.checks.len()
                );
                let outputs = state.outputs;
                let delay_msec = state.delay_msecs;
                let inputs = state.inputs;
                let (next_states, check): (
                    Vec<usize>,
                    BoxedStateCheck<ProcessingResult, HuntResult, InternalHuntState>,
                ) = if !state.checks.is_empty() {
                    let next_states: Vec<usize> =
                        state.checks.keys().map(|x| x + fragment_first).collect();
                    let check = Box::new(
                        move |x: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                            // Vec of InputValue -> (usize, HuntResult)
                            // e.g. Vec<ProcessingResult> -> (usize, StateTransition)
                            let checks = &state.checks;
                            // Mutable result to guarantee all checks run
                            let mut result = None;

                            for (target, func) in checks {
                                if let Some(res) = func(x, int) {
                                    debug_assert!(
                                        result.is_none(),
                                        "Only one state check should match"
                                    );
                                    result = Some((*target + fragment_first, res));
                                }
                            }

                            result
                        },
                    );
                    (next_states, check)
                } else {
                    (
                        vec![next_state],
                        Box::new(
                            move |_: &Vec<ProcessingResult>, _: &mut InternalHuntState| {
                                Some((next_state, HuntResult::default()))
                            },
                        ),
                    )
                };

                fsm.add_state(
                    tag,
                    name,
                    debug_name,
                    outputs,
                    delay_msec,
                    inputs,
                    next_states,
                    check,
                );
            }
            last_index = fragment_last + 1;
            findex += 1;
        }

        HuntFSM::new(fsm)
    }
}

impl<K> StateDescription<K>
where
    K: Hash + Eq + Clone,
{
    pub fn new(
        tag: K,
        inputs: Vec<Processing>,
        buttons: Vec<HuntStateOutput>,
        delay_msecs: Range<u64>,
        check: HashMap<K, BoxedProcessFn>,
    ) -> Self {
        StateDescription {
            tag,
            inputs,
            buttons,
            delay_msecs,
            check,
        }
    }

    pub fn set_counter_state(tag: K, to: K, value: usize) -> Self {
        let mut set_count_check: HashMap<K, BoxedProcessFn> = HashMap::new();

        set_count_check.insert(
            to,
            Box::new(move |_, int| {
                int.counter = value;
                Some(HuntResult::default())
            }),
        );

        Self::new(tag, vec![], vec![], 0..0, set_count_check)
    }

    pub fn choose_counter_state(tag: K, zero: K, nonzero: K) -> Self {
        let mut count_check: HashMap<K, BoxedProcessFn> = HashMap::new();

        count_check.insert(
            zero,
            Box::new(|_, int| {
                if int.counter == 0 {
                    Some(HuntResult::default())
                } else {
                    None
                }
            }),
        );
        count_check.insert(
            nonzero,
            Box::new(|_, int| {
                if int.counter != 0 {
                    Some(HuntResult::default())
                } else {
                    None
                }
            }),
        );

        Self::new(tag, vec![], vec![], 0..0, count_check)
    }

    pub fn decr_counter_state(tag: K, to: K) -> Self {
        let mut count_check: HashMap<K, BoxedProcessFn> = HashMap::new();
        count_check.insert(
            to,
            Box::new(|_: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                int.counter -= 1;

                Some(HuntResult::default())
            }),
        );

        Self::new(tag, vec![], vec![], 0..0, count_check)
    }

    pub fn choose_toggle_state(tag: K, set: K, clear: K) -> Self {
        let mut toggle_check: HashMap<K, BoxedProcessFn> = HashMap::new();

        toggle_check.insert(
            set,
            Box::new(|_: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                if int.toggle {
                    Some(HuntResult::default())
                } else {
                    None
                }
            }),
        );
        toggle_check.insert(
            clear,
            Box::new(|_: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                if int.toggle {
                    None
                } else {
                    Some(HuntResult::default())
                }
            }),
        );

        Self::new(tag, vec![], vec![], 0..0, toggle_check)
    }

    pub fn toggle_state(tag: K, to: K) -> Self {
        let mut toggle_check: HashMap<K, BoxedProcessFn> = HashMap::new();
        toggle_check.insert(
            to,
            Box::new(|_: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                int.toggle = !int.toggle;

                Some(HuntResult::default())
            }),
        );

        Self::new(tag, vec![], vec![], 0..0, toggle_check)
    }

    pub fn linear_state_no_delay(tag: K, buttons: Vec<HuntStateOutput>) -> Self {
        Self::linear_state(tag, buttons, 0..0)
    }

    pub fn linear_state(tag: K, buttons: Vec<HuntStateOutput>, delay_msecs: Range<u64>) -> Self {
        Self::new(tag, Vec::new(), buttons, delay_msecs, HashMap::new())
    }

    pub fn deadend_state(tag: K) -> Self {
        let mut deadend_checks: HashMap<K, BoxedProcessFn> = HashMap::new();
        deadend_checks.insert(
            tag.clone(),
            Box::new(|_res: &Vec<ProcessingResult>, _: &mut InternalHuntState| {
                Some(HuntResult::default())
            }),
        );
        Self::new(tag, Vec::new(), Vec::new(), 0..0, deadend_checks)
    }

    fn simple_process_state_helper(
        tag: K,
        to_met: K,
        to_not: K,
        processing: Processing,
        output: Vec<HuntStateOutput>,
        delay_msecs: Range<u64>,
        start_timer: bool,
        end_timer: bool,
    ) -> Self {
        let mut process_checks: HashMap<K, BoxedProcessFn> = HashMap::new();
        let proc_for_met = processing.clone();
        let proc_for_not = processing.clone();
        process_checks.insert(
            to_met,
            Box::new(
                move |res: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                    if res
                        .iter()
                        .filter(|f| f.process == proc_for_met)
                        .any(|f| f.met)
                    {
                        if start_timer {
                            int.time = SystemTime::now()
                        }
                        if end_timer {
                            int.last_duration = int.time.elapsed().expect("Couldn't get duration")
                        }
                        Some(HuntResult::default())
                    } else {
                        None
                    }
                },
            ),
        );
        process_checks.insert(
            to_not,
            Box::new(
                move |res: &Vec<ProcessingResult>, _: &mut InternalHuntState| {
                    if !res
                        .iter()
                        .filter(|f| f.process == proc_for_not)
                        .any(|f| f.met)
                    {
                        Some(HuntResult::default())
                    } else {
                        None
                    }
                },
            ),
        );
        StateDescription::new(tag, vec![processing], output, delay_msecs, process_checks)
    }

    pub fn simple_process_state_no_output(tag: K, to_met: K, processing: Processing) -> Self {
        Self::simple_process_state_helper(
            tag.clone(),
            to_met,
            tag,
            processing,
            vec![],
            0..0,
            false,
            false,
        )
    }

    pub fn simple_process_state_no_output_start_timer(
        tag: K,
        to_met: K,
        processing: Processing,
    ) -> Self {
        Self::simple_process_state_helper(
            tag.clone(),
            to_met,
            tag,
            processing,
            vec![],
            0..0,
            true,
            false,
        )
    }

    pub fn simple_process_state_no_output_end_timer(
        tag: K,
        to_met: K,
        processing: Processing,
    ) -> Self {
        Self::simple_process_state_helper(
            tag.clone(),
            to_met,
            tag,
            processing,
            vec![],
            0..0,
            false,
            true,
        )
    }

    pub fn simple_process_state(
        tag: K,
        to_met: K,
        to_not: K,
        processing: Processing,
        output: HuntStateOutput,
        delay_msecs: Range<u64>,
    ) -> Self {
        Self::simple_process_state_helper(
            tag,
            to_met,
            to_not,
            processing,
            vec![output],
            delay_msecs,
            false,
            false,
        )
    }

    pub fn simple_sprite_state(
        tag: K,
        to_met: K,
        to_not: K,
        game: &Game,
        method: &Method,
        species: u32,
        target: u32,
    ) -> Self {
        let mut detect_checks: HashMap<K, BoxedProcessFn> = HashMap::new();
        let detect = Processing::Sprite(game.clone(), vec![species], false);

        let shiny_closure = move |res: &Vec<ProcessingResult>| {
            let sprite_results: Vec<&ProcessingResult> = res
                .iter()
                .filter(|r| matches!(r.process, Processing::Sprite(_, _, _)))
                .collect();
            debug_assert_eq!(sprite_results.len(), 1, "Must have a single sprite result");
            let sprite_result = *sprite_results
                .first()
                .expect("Must have a single sprite result");

            let shiny_sprite = sprite_result.shiny;

            log::info!("Detect results: shiny_sprite = {}", shiny_sprite);

            (shiny_sprite, sprite_result.species)
        };

        let game_copy = game.clone();
        let method_copy = method.clone();
        detect_checks.insert(
            to_met,
            Box::new(move |res, _| {
                let (shiny, found_species) = shiny_closure(res);

                if shiny {
                    if found_species == target {
                        Some(HuntResult {
                            transition: Some(RequestTransition {
                                transition: Transition::FoundTarget,
                                arg: None,
                            }),
                            incr_encounters: true,
                        })
                    } else {
                        Some(HuntResult {
                            transition: Some(RequestTransition {
                                transition: Transition::FoundNonTarget,
                                arg: Some(TransitionArg {
                                    name: String::from(""),
                                    species: found_species,
                                    game: game_copy.clone(),
                                    method: method_copy.clone(),
                                }),
                            }),
                            incr_encounters: true,
                        })
                    }
                } else {
                    None
                }
            }),
        );
        detect_checks.insert(
            to_not,
            Box::new(move |res, _| {
                let (shiny, _) = shiny_closure(res);

                if shiny {
                    None
                } else {
                    Some(HuntResult {
                        transition: None,
                        incr_encounters: true,
                    })
                }
            }),
        );

        StateDescription::new(tag, vec![detect], vec![], 0..0, detect_checks)
    }

    pub fn sprite_state_delay(
        tag: K,
        to_met: K,
        to_not: K,
        game: &Game,
        method: &Method,
        species: u32,
        target: u32,
        threshold: Duration,
    ) -> Self {
        let mut detect_checks: HashMap<K, BoxedProcessFn> = HashMap::new();
        let detect = Processing::Sprite(game.clone(), vec![species], false);

        let shiny_closure = move |res: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
            let sprite_results: Vec<&ProcessingResult> = res
                .iter()
                .filter(|r| matches!(r.process, Processing::Sprite(_, _, _)))
                .collect();
            debug_assert_eq!(sprite_results.len(), 1, "Must have a single sprite result");
            let sprite_result = *sprite_results
                .first()
                .expect("Must have a single sprite result");

            let shiny_sprite = sprite_result.shiny;
            let shiny_duration = int.last_duration > threshold;

            log::info!(
                "Detect results: shiny_sprite = {}, shiny_duration = {} ({:?}/{:?})",
                shiny_sprite,
                shiny_duration,
                int.last_duration,
                threshold
            );

            (shiny_sprite | shiny_duration, sprite_result.species)
        };

        let game_copy = game.clone();
        let method_copy = method.clone();
        detect_checks.insert(
            to_met,
            Box::new(move |res, int| {
                let (shiny, found_species) = shiny_closure(res, int);

                if shiny {
                    if found_species == target {
                        Some(HuntResult {
                            transition: Some(RequestTransition {
                                transition: Transition::FoundTarget,
                                arg: None,
                            }),
                            incr_encounters: true,
                        })
                    } else {
                        Some(HuntResult {
                            transition: Some(RequestTransition {
                                transition: Transition::FoundNonTarget,
                                arg: Some(TransitionArg {
                                    name: String::from(""),
                                    species: found_species,
                                    game: game_copy.clone(),
                                    method: method_copy.clone(),
                                }),
                            }),
                            incr_encounters: true,
                        })
                    }
                } else {
                    None
                }
            }),
        );
        detect_checks.insert(
            to_not,
            Box::new(move |res, int| {
                let (shiny, _) = shiny_closure(res, int);

                if shiny {
                    None
                } else {
                    Some(HuntResult {
                        transition: None,
                        incr_encounters: true,
                    })
                }
            }),
        );

        StateDescription::new(tag, vec![detect], vec![], 0..0, detect_checks)
    }
}
