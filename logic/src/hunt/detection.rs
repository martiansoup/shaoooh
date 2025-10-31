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
    PreEnterEncounter,
    EnterEncounter,
    WaitEncounterReady,
    PressA,
    Detect,
    Done,
    Run1,
    Run2,
    Run3,
    Run4,
    Run5,
    Run6,
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

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum CheckSummary {
    Start,
    Down,
    ToPokemon,
    Up1,
    Up2,
    Select,
    ToSummary,
    Detect,
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
        } else if *game == Game::RubySapphire && *method == Method::SoftResetGift {
            Self::gen3_softreset_gift(builder)
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
        } else if *game == Game::HeartGoldSoulSilver
            && *method == Method::SoftResetEncounter
            && builder.target() == 206
        {
            Some(Self::hgss_darkcave(builder))
        } else {
            None
        }
    }

    pub fn gen3_softreset_gift(mut builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let target = builder.target();
        let game = builder.game().clone();
        let method = builder.method().clone();

        let states = vec![
            StateDescription::linear_state(
                CheckSummary::Start,
                vec![HuntStateOutput::button(Button::Start)],
                500..500,
            ),
            StateDescription::linear_state(
                CheckSummary::Down,
                vec![HuntStateOutput::button(Button::Down)],
                250..250,
            ),
            StateDescription::linear_state(
                CheckSummary::ToPokemon,
                vec![HuntStateOutput::button(Button::A)],
                1000..1000,
            ),
            StateDescription::linear_state(
                CheckSummary::Up1,
                vec![HuntStateOutput::button(Button::Up)],
                250..250,
            ),
            StateDescription::linear_state(
                CheckSummary::Up2,
                vec![HuntStateOutput::button(Button::Up)],
                250..250,
            ),
            StateDescription::linear_state(
                CheckSummary::Select,
                vec![HuntStateOutput::button(Button::A)],
                500..500,
            ),
            StateDescription::linear_state(
                CheckSummary::ToSummary,
                vec![HuntStateOutput::button(Button::A)],
                1000..1000,
            ),
            StateDescription::simple_sprite_state_flip(
                Branch3::new(
                    CheckSummary::Detect,
                    CheckSummary::Done,
                    CheckSummary::NextAttempt,
                ),
                &game,
                &method,
                target,
                target,
                true,
            ),
            StateDescription::deadend_state(CheckSummary::Done),
            StateDescription::linear_state(CheckSummary::NextAttempt, vec![], 500..5000),
        ];

        builder.add_states(states);
        Some(builder)
    }

    pub fn hgss_darkcave(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let duration = 6800;
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
                Processing::HGSS_ENCOUNTER_READY,
            ),
            StateDescription::sprite_state_delay_targets(
                Branch3::new(Detection::Detect, Detection::Done, Detection::Run1),
                game,
                method,
                vec![206, 74],
                species,
                shiny_threshold,
            ),
            StateDescription::deadend_state(Detection::Done),
            StateDescription::linear_state(
                Detection::Run1,
                vec![HuntStateOutput::button(Button::Down)],
                1000..1000,
            ),
            StateDescription::linear_state(
                Detection::Run2,
                vec![HuntStateOutput::button(Button::Down)],
                1000..1000,
            ),
            StateDescription::linear_state(
                Detection::Run3,
                vec![HuntStateOutput::button(Button::Right)],
                1000..1000,
            ),
            StateDescription::linear_state(
                Detection::Run4,
                vec![HuntStateOutput::button(Button::Right)],
                1000..1000,
            ),
            StateDescription::linear_state(
                Detection::Run5,
                vec![HuntStateOutput::button(Button::Left)],
                1000..1000,
            ),
            StateDescription::linear_state(
                Detection::Run6,
                vec![HuntStateOutput::button(Button::A)],
                10000..10000,
            ),
        ];

        builder.add_states(states);
        builder
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
        let timer = match species {
            144 => 16750, // Articuno
            145 => 16750, // Zapdos
            146 => 16750, // Moltres
            244 => 16650, // Entei
            382 => 16975, // Kyogre
            380 => 10500, // Latias
            717 => 16750, // Yveltal
            799 => 100, // Guzzlord
            _ => 100,
        };

        let states = vec![
            StateDescription::simple_process_state_no_output(
                Branch2::new(Detection::PreEnterEncounter, Detection::EnterEncounter),
                Processing::USUMBottomScreenInv(5.0),
            ),
            StateDescription::simple_process_state_no_output_start_timer(
                Branch2::new(Detection::EnterEncounter, Detection::Detect),
                Processing::USUMBottomScreen(5.0),
            ),
            StateDescription::simple_process_state_no_output_end_timer(
                Branch2::new(Detection::Detect, Detection::Run1),
                Processing::USUMBottomScreen(60.0),
            ),
            StateDescription::branch_last_delay_state(
                Branch3::new(Detection::Run1, Detection::Toggle, Detection::Run2),
                timer,
            ),
            StateDescription::found_target_state(Detection::Toggle, Detection::Done),
            StateDescription::deadend_state(Detection::Done),
            StateDescription::incr_encounter_state(Detection::Run2, Detection::Run3),
            StateDescription::clear_atomic_state(Detection::Run3, Detection::Run4),
            StateDescription::linear_state(Detection::Run4, vec![], 0..500),
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
