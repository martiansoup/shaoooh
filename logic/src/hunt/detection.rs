use std::{collections::HashMap, time::Duration};

use strum_macros::AsRefStr;

use crate::{
    app::{Game, Method, RequestTransition, Transition, TransitionArg},
    control::{Button, Delay},
    hunt::{
        BoxedProcessFn, HuntFSMBuilder, HuntResult, HuntStateOutput, InternalHuntState,
        StateDescription,
    },
    vision::{Processing, ProcessingResult},
};

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum Detection {
    EnterEncounter,
    WaitEncounterReady,
    PressA,
    Detect,
    Done,
    Run1,
    Run2,
    Run3,
    Run4,
    Toggle,
}

pub struct DetectionResolver {}

impl DetectionResolver {
    pub fn add_states(builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let game = builder.game();
        let method = builder.method();
        log::info!("Adding Encounter Type states for '{:?}/{:?}'", game, method);

        if *game == Game::FireRedLeafGreen && *method == Method::RandomEncounter {
            Some(Self::frlg_random(builder))
        } else {
            None
        }
    }

    pub fn frlg_random(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        // TODO detection state builder
        let shiny_threshold = Duration::from_millis(3250);
        // TODO hardcoded for Route 1
        let detect = Processing::Sprite(Game::FireRedLeafGreen, vec![16, 19], false);
        let mut detect_checks: HashMap<Detection, BoxedProcessFn> = HashMap::new();
        let target = builder.target();
        let game = builder.game().clone();
        let method = builder.method().clone();
        let shiny_closure = move |res: &Vec<ProcessingResult>, int: &mut InternalHuntState| {
            let sprite_results: Vec<&ProcessingResult> = res
                .iter()
                .filter(|r| matches!(r.process, Processing::Sprite(_, _, _)))
                .collect();
            assert_eq!(sprite_results.len(), 1, "Must have a single sprite result");
            let sprite_result = *sprite_results
                .first()
                .expect("Must have a single sprite result");

            let shiny_sprite = sprite_result.shiny;
            let shiny_duration = int.last_duration > shiny_threshold;

            log::info!(
                "Detect results: shiny_sprite = {}, shiny_duration = {} ({:?}/{:?})",
                shiny_sprite,
                shiny_duration,
                int.last_duration,
                shiny_threshold
            );

            (shiny_sprite | shiny_duration, sprite_result.species)
        };

        detect_checks.insert(
            Detection::Done,
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
                                    game: game.clone(),
                                    method: method.clone(),
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
            Detection::Run1,
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

        let states = vec![
            StateDescription::toggle_state(Detection::Toggle, Detection::EnterEncounter),
            StateDescription::simple_process_state_no_output_start_timer(
                Detection::EnterEncounter,
                Detection::WaitEncounterReady,
                Processing::FRLG_IN_ENCOUNTER,
            ),
            StateDescription::simple_process_state_no_output_end_timer(
                Detection::WaitEncounterReady,
                Detection::PressA,
                Processing::FRLG_ENCOUNTER_READY,
            ),
            StateDescription::linear_state(
                Detection::PressA,
                vec![HuntStateOutput::new(Button::A, Delay::Tenth)],
                5000..5000,
            ),
            // Detect
            StateDescription::new(Detection::Detect, vec![detect], vec![], 0..0, detect_checks),
            // Done
            StateDescription::deadend_state(Detection::Done),
            // Run
            StateDescription::linear_state(
                Detection::Run1,
                vec![HuntStateOutput::new(Button::Down, Delay::Tenth)],
                1000..1000,
            ),
            StateDescription::linear_state(
                Detection::Run2,
                vec![HuntStateOutput::new(Button::Right, Delay::Tenth)],
                1000..1000,
            ),
            StateDescription::linear_state(
                Detection::Run3,
                vec![HuntStateOutput::new(Button::A, Delay::Tenth)],
                1000..1000,
            ),
            StateDescription::linear_state(
                Detection::Run4,
                vec![HuntStateOutput::new(Button::A, Delay::Tenth)],
                3000..3000,
            ),
        ];

        builder.add_states(states);
        builder
    }
}
