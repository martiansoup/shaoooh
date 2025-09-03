use std::{collections::HashMap, time::Duration};

use strum_macros::AsRefStr;

use crate::{
    app::{Game, Method, RequestTransition, Transition, TransitionArg},
    control::{Button, Delay},
    hunt::{
        BoxedProcessFn, Branch2, Branch3, HuntFSMBuilder, HuntResult, HuntStateOutput,
        InternalHuntState, StateDescription,
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

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum HGSSStarter {
    Left1,
    Detect155,
    Left2,
    Detect158,
    Left3,
    Detect152,
    Done,
    NextAttempt,
}

pub struct DetectionResolver {}

impl DetectionResolver {
    pub fn add_states(builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let game = builder.game();
        let method = builder.method();
        log::info!("Adding Detection states for '{:?}/{:?}'", game, method);

        if *game == Game::FireRedLeafGreen && *method == Method::RandomEncounter {
            Some(Self::frlg_random(builder))
        } else if *game == Game::FireRedLeafGreen && *method == Method::SoftResetEncounter {
            Some(Self::frlg_softreset(builder))
        } else if *game == Game::HeartGoldSoulSilver && *method == Method::SoftResetGift {
            match builder.target() {
                152 | 155 | 158 => Some(Self::hgss_starter(builder)),
                _ => None,
            }
        } else if *game == Game::DiamondPearl && *method == Method::SoftResetEncounter {
            match builder.target() {
                486 | 487 => Some(Self::gen4_legend(builder)),
                _ => None,
            }
        } else if *game == Game::UltraSunUltraMoon && *method == Method::SoftResetEncounter {
            Some(Self::gen7_legend(builder))
        } else {
            None
        }
    }

    pub fn hgss_starter(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let game = builder.game();
        let method = builder.method();

        let states = vec![
            StateDescription::linear_state(
                HGSSStarter::Left1,
                vec![HuntStateOutput::button(Button::Left)],
                1000..1500,
            ),
            StateDescription::simple_sprite_state(
                Branch3::new(
                    HGSSStarter::Detect155,
                    HGSSStarter::Done,
                    HGSSStarter::Left2,
                ),
                game,
                method,
                155,
                builder.target(),
            ),
            StateDescription::linear_state(
                HGSSStarter::Left2,
                vec![HuntStateOutput::button(Button::Left)],
                1000..1500,
            ),
            StateDescription::simple_sprite_state(
                Branch3::new(
                    HGSSStarter::Detect158,
                    HGSSStarter::Done,
                    HGSSStarter::Left3,
                ),
                game,
                method,
                158,
                builder.target(),
            ),
            StateDescription::linear_state(
                HGSSStarter::Left3,
                vec![HuntStateOutput::button(Button::Left)],
                1000..1500,
            ),
            StateDescription::simple_sprite_state(
                Branch3::new(
                    HGSSStarter::Detect152,
                    HGSSStarter::Done,
                    HGSSStarter::NextAttempt,
                ),
                game,
                method,
                152,
                builder.target(),
            ),
            StateDescription::deadend_state(HGSSStarter::Done),
            StateDescription::linear_state(HGSSStarter::NextAttempt, vec![], 0..2000),
        ];

        builder.add_states(states);
        builder
    }

    pub fn gen4_legend(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let duration = match builder.target() {
            486 => 7750, // Regigigas
            487 => 6600, // Giratina
            _ => 5000,
        };
        let shiny_threshold = Duration::from_millis(duration);
        let game = builder.game();
        let method = builder.method();
        let species = builder.target();

        let states = vec![
            StateDescription::simple_process_state_no_output_start_timer(
                Branch2::new(Detection::EnterEncounter, Detection::WaitEncounterReady),
                Processing::DP_IN_ENCOUNTER,
            ),
            StateDescription::simple_process_state_no_output_end_timer(
                Branch2::new(Detection::WaitEncounterReady, Detection::Detect),
                Processing::DP_ENCOUNTER_READY,
            ),
            StateDescription::sprite_state_delay(
                Branch3::new(Detection::Detect, Detection::Done, Detection::Run1),
                game,
                method,
                species,
                species,
                shiny_threshold,
            ),
            StateDescription::deadend_state(Detection::Done),
            StateDescription::linear_state(Detection::Run1, vec![], 1000..3000),
        ];

        builder.add_states(states);
        builder
    }

    pub fn gen7_legend(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let game = builder.game();
        let method = builder.method();
        let species = builder.target();

        let states = vec![
            StateDescription::start_timer_state(Detection::EnterEncounter, Detection::Detect),
            StateDescription::simple_process_state_no_output3(
                Branch3::new(Detection::Detect, Detection::Toggle, Detection::PressA),
                Processing::USUMShinyStar(species),
            ),
            StateDescription::branch_delay_state(
                Branch3::new(
                    Detection::PressA,
                    Detection::Run1,
                    Detection::WaitEncounterReady,
                ),
                14000,
            ),
            StateDescription::branch_state(
                Detection::WaitEncounterReady,
                Detection::Detect,
                50..50,
            ),
            StateDescription::found_target_state(Detection::Toggle, Detection::Done),
            StateDescription::deadend_state(Detection::Done),
            StateDescription::incr_encounter_state(Detection::Run1, Detection::Run2),
            StateDescription::linear_state(Detection::Run2, vec![], 0..500),
        ];

        builder.add_states(states);
        builder
    }

    pub fn frlg_softreset(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let shiny_threshold = Duration::from_millis(2400);
        let target = builder.target();
        let game = builder.game().clone();
        let method = builder.method().clone();

        let states = vec![
            StateDescription::simple_process_state_no_output_start_timer(
                Branch2::new(Detection::EnterEncounter, Detection::WaitEncounterReady),
                Processing::FRLG_IN_ENCOUNTER,
            ),
            StateDescription::simple_process_state_no_output_end_timer(
                Branch2::new(Detection::WaitEncounterReady, Detection::PressA),
                Processing::FRLG_ENCOUNTER_READY,
            ),
            StateDescription::linear_state(
                Detection::PressA,
                vec![HuntStateOutput::new(Button::A, Delay::Tenth)],
                3000..3000,
            ),
            StateDescription::sprite_state_delay(
                Branch3::new(Detection::Detect, Detection::Done, Detection::Run1),
                &game,
                &method,
                target,
                target,
                shiny_threshold,
            ),
            StateDescription::deadend_state(Detection::Done),
            StateDescription::linear_state(Detection::Run1, vec![], 1000..3000),
        ];

        builder.add_states(states);
        builder
    }

    const RUN_DELAY: u64 = 500;

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
            debug_assert_eq!(sprite_results.len(), 1, "Must have a single sprite result");
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
                Branch2::new(Detection::EnterEncounter, Detection::WaitEncounterReady),
                Processing::FRLG_IN_ENCOUNTER,
            ),
            StateDescription::simple_process_state_no_output_end_timer(
                Branch2::new(Detection::WaitEncounterReady, Detection::PressA),
                Processing::FRLG_ENCOUNTER_READY,
            ),
            StateDescription::linear_state(
                Detection::PressA,
                vec![HuntStateOutput::new(Button::A, Delay::Tenth)],
                3000..3000,
            ),
            // Detect
            StateDescription::new(Detection::Detect, vec![detect], vec![], 0..0, detect_checks),
            // Done
            StateDescription::deadend_state(Detection::Done),
            // Run
            StateDescription::linear_state(
                Detection::Run1,
                vec![HuntStateOutput::new(Button::Down, Delay::Tenth)],
                Self::RUN_DELAY..Self::RUN_DELAY,
            ),
            StateDescription::linear_state(
                Detection::Run2,
                vec![HuntStateOutput::new(Button::Right, Delay::Tenth)],
                Self::RUN_DELAY..Self::RUN_DELAY,
            ),
            StateDescription::linear_state(
                Detection::Run3,
                vec![HuntStateOutput::new(Button::A, Delay::Tenth)],
                Self::RUN_DELAY..Self::RUN_DELAY,
            ),
            StateDescription::linear_state(
                Detection::Run4,
                vec![HuntStateOutput::new(Button::A, Delay::Tenth)],
                3000..7200,
            ),
        ];

        builder.add_states(states);
        builder
    }
}
