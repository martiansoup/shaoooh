use crate::app::{Game, Method, RequestTransition, Transition, TransitionArg};
use crate::fsm::{BoxedStateCheck, StateId, StateMachine};
use crate::hunt::state_machine::HuntStateOutput;
use crate::hunt::{BaseHunt, HuntFSM, HuntResult, InternalHuntState};
use crate::vision::{Processing, ProcessingResult};
use std::cmp::Eq;
use std::collections::HashMap;
use std::convert::AsRef;
use std::hash::Hash;
use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

pub type BoxedProcessFn =
    Box<dyn Fn(&Vec<ProcessingResult>, &mut InternalHuntState) -> Option<HuntResult>>;

pub struct Branch2<K> {
    tag: K,
    to_met: K,
}

pub struct Branch3<K> {
    tag: K,
    to_met: K,
    to_not: K,
}

impl<K> Branch2<K> {
    pub fn new(tag: K, to_met: K) -> Self {
        Branch2 { tag, to_met }
    }
}

impl<K> Branch3<K> {
    pub fn new(tag: K, to_met: K, to_not: K) -> Self {
        Branch3 {
            tag,
            to_met,
            to_not,
        }
    }
}

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

    pub fn build(self, atomic: Arc<AtomicBool>) -> HuntFSM {
        let mut last_index = 0;
        let mut fsm = StateMachine::new(InternalHuntState::new(atomic));

        let last_fragment = self.fragments.len() - 1;
        for (findex, fragment) in self.fragments.into_iter().enumerate() {
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
                let name = &state.name;
                let debug_name = format!(
                    "fragment{}-{} [{} next state(s)]",
                    findex,
                    name,
                    state.checks.len()
                );
                let outputs = &state.outputs;
                let delay_msec = &state.delay_msecs;
                let inputs = &state.inputs;
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
                    StateId::new(tag, name.to_string(), debug_name),
                    outputs.to_vec(),
                    delay_msec.clone(),
                    inputs.to_vec(),
                    next_states,
                    check,
                );
            }
            last_index = fragment_last + 1;
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

    pub fn branch_state(tag: K, to: K, delay: Range<u64>) -> Self {
        let mut branch_state: HashMap<K, BoxedProcessFn> = HashMap::new();

        branch_state.insert(to, Box::new(move |_, _| Some(HuntResult::default())));

        Self::new(tag, vec![], vec![], delay, branch_state)
    }

    pub fn start_timer_state(tag: K, to: K) -> Self {
        let mut branch_state: HashMap<K, BoxedProcessFn> = HashMap::new();

        branch_state.insert(
            to,
            Box::new(move |_, int| {
                int.time = SystemTime::now();
                Some(HuntResult::default())
            }),
        );

        Self::new(tag, vec![], vec![], 0..0, branch_state)
    }

    pub fn update_timer_state(tag: K, to: K) -> Self {
        let mut branch_state: HashMap<K, BoxedProcessFn> = HashMap::new();

        branch_state.insert(
            to,
            Box::new(move |_, int| {
                int.last_duration = int.time.elapsed().unwrap();
                Some(HuntResult::default())
            }),
        );

        Self::new(tag, vec![], vec![], 0..0, branch_state)
    }

    pub fn branch_delay_state(branch: Branch3<K>, delay: u64) -> Self {
        let mut branch_check: HashMap<K, BoxedProcessFn> = HashMap::new();
        let duration = Duration::from_millis(delay);
        let dur2 = duration.clone();

        branch_check.insert(
            branch.to_met,
            Box::new(
                move |_: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                    if int.time.elapsed().unwrap() > duration {
                        Some(HuntResult::default())
                    } else {
                        None
                    }
                },
            ),
        );
        branch_check.insert(
            branch.to_not,
            Box::new(
                move |_: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                    if int.time.elapsed().unwrap() > dur2 {
                        None
                    } else {
                        Some(HuntResult::default())
                    }
                },
            ),
        );

        Self::new(branch.tag, vec![], vec![], 0..0, branch_check)
    }

    pub fn branch_last_delay_state(branch: Branch3<K>, delay: u64) -> Self {
        let mut branch_check: HashMap<K, BoxedProcessFn> = HashMap::new();
        let duration = Duration::from_millis(delay);
        let dur2 = duration.clone();

        branch_check.insert(
            branch.to_met,
            Box::new(
                move |_: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                    if int.last_duration > duration {
                        log::info!("Timer met ({:?} > {:?})", int.last_duration, duration);
                        Some(HuntResult::default())
                    } else {
                        None
                    }
                },
            ),
        );
        branch_check.insert(
            branch.to_not,
            Box::new(
                move |_: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                    if int.last_duration > dur2 {
                        None
                    } else {
                        log::info!("Timer not met ({:?} < {:?})", int.last_duration, dur2);
                        Some(HuntResult::default())
                    }
                },
            ),
        );

        Self::new(branch.tag, vec![], vec![], 0..0, branch_check)
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

    pub fn choose_counter_state_val(tag: K, equal: K, nonequal: K, value: usize) -> Self {
        let mut count_check: HashMap<K, BoxedProcessFn> = HashMap::new();

        count_check.insert(
            equal,
            Box::new(move |_, int| {
                if int.counter == value {
                    Some(HuntResult::default())
                } else {
                    None
                }
            }),
        );
        count_check.insert(
            nonequal,
            Box::new(move |_, int| {
                if int.counter != value {
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

    pub fn incr_counter_state(tag: K, to: K) -> Self {
        let mut count_check: HashMap<K, BoxedProcessFn> = HashMap::new();
        count_check.insert(
            to,
            Box::new(|_: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                int.counter += 1;

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

    pub fn set_atomic_state(tag: K, to: K) -> Self {
        let mut toggle_check: HashMap<K, BoxedProcessFn> = HashMap::new();
        toggle_check.insert(
            to,
            Box::new(|_: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                int.atomic.store(true, Ordering::Release);

                Some(HuntResult::default())
            }),
        );

        Self::new(tag, vec![], vec![], 0..0, toggle_check)
    }

    pub fn clear_atomic_state(tag: K, to: K) -> Self {
        let mut toggle_check: HashMap<K, BoxedProcessFn> = HashMap::new();
        toggle_check.insert(
            to,
            Box::new(|_: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
                int.atomic.store(false, Ordering::Release);

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
        branch: Branch3<K>,
        processing: Processing,
        output: Vec<HuntStateOutput>,
        delay_msecs: Range<u64>,
        start_timer: bool,
        end_timer: bool,
    ) -> Self {
        let Branch3 {
            tag,
            to_met,
            to_not,
        } = branch;
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

    pub fn simple_process_state_no_output(branch: Branch2<K>, processing: Processing) -> Self {
        let Branch2 { tag, to_met } = branch;
        Self::simple_process_state_helper(
            Branch3::new(tag.clone(), to_met, tag),
            processing,
            vec![],
            0..0,
            false,
            false,
        )
    }

    pub fn simple_process_state_no_output3(branch: Branch3<K>, processing: Processing) -> Self {
        Self::simple_process_state_helper(branch, processing, vec![], 0..0, false, false)
    }

    pub fn simple_process_state_no_output_start_timer(
        branch: Branch2<K>,
        processing: Processing,
    ) -> Self {
        let Branch2 { tag, to_met } = branch;
        Self::simple_process_state_helper(
            Branch3::new(tag.clone(), to_met, tag),
            processing,
            vec![],
            0..0,
            true,
            false,
        )
    }

    pub fn simple_process_state_no_output_end_timer(
        branch: Branch2<K>,
        processing: Processing,
    ) -> Self {
        let Branch2 { tag, to_met } = branch;
        Self::simple_process_state_helper(
            Branch3::new(tag.clone(), to_met, tag),
            processing,
            vec![],
            0..0,
            false,
            true,
        )
    }

    pub fn simple_process_state(
        branch: Branch3<K>,
        processing: Processing,
        output: HuntStateOutput,
        delay_msecs: Range<u64>,
    ) -> Self {
        Self::simple_process_state_helper(
            branch,
            processing,
            vec![output],
            delay_msecs,
            false,
            false,
        )
    }

    pub fn found_target_state(tag: K, next: K) -> Self {
        let mut detect_checks: HashMap<K, BoxedProcessFn> = HashMap::new();

        detect_checks.insert(
            next,
            Box::new(move |_, _| {
                Some(HuntResult {
                    transition: Some(RequestTransition {
                        transition: Transition::FoundTarget,
                        arg: None,
                    }),
                    incr_encounters: true,
                })
            }),
        );

        Self::new(tag, vec![], vec![], 0..0, detect_checks)
    }

    pub fn incr_encounter_state(tag: K, next: K) -> Self {
        let mut detect_checks: HashMap<K, BoxedProcessFn> = HashMap::new();

        detect_checks.insert(
            next,
            Box::new(move |_, _| {
                Some(HuntResult {
                    transition: None,
                    incr_encounters: true,
                })
            }),
        );

        Self::new(tag, vec![], vec![], 0..0, detect_checks)
    }

    pub fn simple_sprite_state(
        branch: Branch3<K>,
        game: &Game,
        method: &Method,
        species: u32,
        target: u32,
    ) -> Self {
        Self::simple_sprite_state_flip(branch, game, method, species, target, false)
    }

    pub fn simple_sprite_state_flip(
        branch: Branch3<K>,
        game: &Game,
        method: &Method,
        species: u32,
        target: u32,
        flip: bool,
    ) -> Self {
        let Branch3 {
            tag,
            to_met,
            to_not,
        } = branch;
        let mut detect_checks: HashMap<K, BoxedProcessFn> = HashMap::new();
        let detect = Processing::Sprite(game.clone(), vec![species], flip);

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

    pub fn simple_sprite_state_flip_w_star(
        branch: Branch3<K>,
        game: &Game,
        method: &Method,
        species: u32,
        target: u32,
        flip: bool,
    ) -> Self {
        let Branch3 {
            tag,
            to_met,
            to_not,
        } = branch;
        let mut detect_checks: HashMap<K, BoxedProcessFn> = HashMap::new();
        let detect = vec![
            Processing::Sprite(game.clone(), vec![species], flip),
            Processing::FRLG_SHINY_STAR,
        ];

        let shiny_closure = move |res: &Vec<ProcessingResult>| {
            let sprite_results: Vec<&ProcessingResult> = res
                .iter()
                .filter(|r| matches!(r.process, Processing::Sprite(_, _, _)))
                .collect();
            debug_assert_eq!(sprite_results.len(), 1, "Must have a single sprite result");
            let sprite_result = *sprite_results
                .first()
                .expect("Must have a single sprite result");

            let star_results: Vec<&ProcessingResult> = res
                .iter()
                .filter(|r| matches!(r.process, Processing::FRLG_SHINY_STAR))
                .collect();
            debug_assert_eq!(star_results.len(), 1, "Must have a single star result");
            let star_result = *star_results
                .first()
                .expect("Must have a single star result");

            let shiny_sprite = sprite_result.shiny;
            let shiny_star = star_result.met;

            log::info!(
                "Detect results: shiny_sprite = {}, shiny_star = {}",
                shiny_sprite,
                shiny_star
            );

            (shiny_sprite, shiny_star, sprite_result.species)
        };

        let game_copy = game.clone();
        let method_copy = method.clone();
        detect_checks.insert(
            to_met,
            Box::new(move |res, _| {
                let (shiny_sprite, shiny_star, found_species) = shiny_closure(res);

                if shiny_sprite || shiny_star {
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
                let (shiny_sprite, shiny_star, _) = shiny_closure(res);

                if shiny_sprite || shiny_star {
                    None
                } else {
                    Some(HuntResult {
                        transition: None,
                        incr_encounters: true,
                    })
                }
            }),
        );

        StateDescription::new(tag, detect, vec![], 0..0, detect_checks)
    }

    pub fn sprite_state_delay(
        branch: Branch3<K>,
        game: &Game,
        method: &Method,
        species: u32,
        target: u32,
        threshold: Duration,
    ) -> Self {
        let Branch3 {
            tag,
            to_met,
            to_not,
        } = branch;
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

    pub fn sprite_state_delay_targets(
        branch: Branch3<K>,
        game: &Game,
        method: &Method,
        species: Vec<u32>,
        target: u32,
        threshold: Duration,
    ) -> Self {
        let Branch3 {
            tag,
            to_met,
            to_not,
        } = branch;
        let mut detect_checks: HashMap<K, BoxedProcessFn> = HashMap::new();
        let detect = Processing::Sprite(game.clone(), species, false);

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

    pub fn simple_sprite_state_3ds(
        branch: Branch3<K>,
        game: &Game,
        method: &Method,
        species: u32,
        target: u32,
    ) -> Self {
        let Branch3 {
            tag,
            to_met,
            to_not,
        } = branch;
        let mut detect_checks: HashMap<K, BoxedProcessFn> = HashMap::new();
        let detect = Processing::Sprite3DS(game.clone(), vec![species]);

        let shiny_closure = move |res: &Vec<ProcessingResult>| {
            let sprite_results: Vec<&ProcessingResult> = res
                .iter()
                .filter(|r| matches!(r.process, Processing::Sprite3DS(_, _)))
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
}
