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
    PrePreEnterEncounter,
    WaitPrePreEnter,
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

// USUM Gift (Poipole)
#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum StickyState {
    DetectSprite,
    StartMashB,
    MashB,
    CheckTimer,
    OpenMenu,
    ToParty,
    ToLast,
    Select,
    OpenSummary,
    DetectStar,
    FoundTarget,
    Done,
    StopHeartbeat,
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
        } else if *game == Game::RubySapphire && *method == Method::RandomEncounter {
            Some(Self::rs_random(builder))
        } else if *game == Game::FireRedLeafGreen && *method == Method::SoftResetEncounter {
            Some(Self::frlg_softreset(builder))
        } else if (*game == Game::RubySapphire || *game == Game::FireRedLeafGreen)
            && *method == Method::SoftResetGift
        {
            Self::gen3_softreset_gift(builder)
        } else if *game == Game::HeartGoldSoulSilver && *method == Method::SoftResetGift {
            match builder.target() {
                152 | 155 | 158 => Some(Self::hgss_starter(builder)),
                _ => None,
            }
        } else if *game == Game::DiamondPearl && *method == Method::SoftResetEncounter {
            match builder.target() {
                442 | 486 | 487 => Some(Self::gen4_legend(builder)),
                _ => None,
            }
        } else if *game == Game::UltraSunUltraMoon && *method == Method::SoftResetEncounter {
            Some(Self::gen7_legend(builder))
        } else if *game == Game::UltraSunUltraMoon && *method == Method::SoftResetGift {
            Some(Self::gen7_gift(builder))
        } else if *game == Game::UltraSunUltraMoon && *method == Method::RandomEncounter {
            Some(Self::gen7_random_encounter(builder))
        } else if *game == Game::HeartGoldSoulSilver
            && *method == Method::SoftResetEncounter
            && builder.target() == 206
        {
            Some(Self::hgss_darkcave(builder))
        } else if *method == Method::Utility {
            // If Utility, skip by default
            Some(builder)
        } else {
            None
        }
    }

    pub fn gen3_softreset_gift(mut builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let target = builder.target();
        let game = builder.game().clone();
        let method = builder.method().clone();
        let base = if game == Game::RubySapphire { 250 } else { 500 };

        let states = vec![
            StateDescription::linear_state(
                CheckSummary::Start,
                vec![HuntStateOutput::button(Button::Start)],
                (base * 2)..(base * 2),
            ),
            StateDescription::linear_state(
                CheckSummary::Down,
                vec![HuntStateOutput::button(Button::Down)],
                (base * 1)..(base * 1),
            ),
            StateDescription::linear_state(
                CheckSummary::ToPokemon,
                vec![HuntStateOutput::button(Button::A)],
                (base * 4)..(base * 4),
            ),
            StateDescription::linear_state(
                CheckSummary::Up1,
                vec![HuntStateOutput::button(Button::Up)],
                (base * 1)..(base * 1),
            ),
            StateDescription::linear_state(
                CheckSummary::Up2,
                vec![HuntStateOutput::button(Button::Up)],
                (base * 1)..(base * 1),
            ),
            StateDescription::linear_state(
                CheckSummary::Select,
                vec![HuntStateOutput::button(Button::A)],
                (base * 2)..(base * 2),
            ),
            StateDescription::linear_state(
                CheckSummary::ToSummary,
                vec![HuntStateOutput::button(Button::A)],
                (base * 4)..(base * 4),
            ),
        ];

        builder.add_states(states);

        if builder.game() == &Game::RubySapphire {
            let states2 = vec![
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

            builder.add_states(states2);
        } else if builder.game() == &Game::FireRedLeafGreen {
            let states2 = vec![
                StateDescription::simple_sprite_state_flip_w_star(
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

            builder.add_states(states2);
        } else {
            return None;
        }

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
            442 => 8500, // Spiritomb
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
        let species = builder.target();
        let timer = match species {
            144 => 16750, // Articuno
            145 => 16750, // Zapdos
            146 => 16750, // Moltres
            243 => 13350, // Raikou
            244 => 16650, // Entei
            245 => 13350, // Suicune
            382 => 16975, // Kyogre
            383 => 15700, // Groudon
            380 => 10500, // Latias
            717 => 16750, // Yveltal
            797 => 20460, // Celesteela
            799 => 21070, // Guzzlord
            806 => 16390, // Blacephalon
            249 => 13610, // Lugia
            643 => 13370, // Reshiram
            644 => 14400, // Zekrom
            488 => 10400, // Cresselia
            150 => 13750, // Mewtwo
            _ => 100,
        };

        if builder.target() == 806 {
            let pre_states = vec![
                StateDescription::simple_process_state_no_output(
                    Branch2::new(Detection::PrePreEnterEncounter, Detection::WaitPrePreEnter),
                    Processing::USUMBottomScreenInv(5.0),
                ),
                StateDescription::simple_process_state_no_output(
                    Branch2::new(Detection::WaitPrePreEnter, Detection::PreEnterEncounter),
                    Processing::USUMBottomScreen(5.0),
                ),
                StateDescription::linear_state_no_delay(Detection::PreEnterEncounter, vec![]),
            ];
            builder.add_states(pre_states);

            let states = vec![
                // TODO only check bottom screen for UBs? avoid getting stuck on missed input
                // or add timeout to this
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
                StateDescription::branch_last_delay_state_plus_range(
                    Branch3::new(Detection::Run1, Detection::Toggle, Detection::Run2),
                    timer,
                    10500..15000,
                ),
                StateDescription::found_target_state(Detection::Toggle, Detection::Done),
                StateDescription::deadend_state(Detection::Done),
                StateDescription::incr_encounter_state(Detection::Run2, Detection::Run3),
                StateDescription::clear_atomic_state(Detection::Run3, Detection::Run4),
                StateDescription::linear_state(Detection::Run4, vec![], 0..500),
            ];

            builder.add_states(states);
        } else {
            let states = vec![
                // TODO only check bottom screen for UBs? avoid getting stuck on missed input
                // or add timeout to this
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
        }

        builder
    }

    pub fn frlg_softreset(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let shiny_threshold = Duration::from_millis(2385);
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

    pub fn rs_random(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let target = builder.target();
        let game = builder.game().clone();
        let method = builder.method().clone();
        let shiny_threshold = Duration::from_millis(2381);

        let targets = if target == 327 {
            vec![327, 27, 227]
        } else {
            vec![target]
        };

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
            StateDescription::sprite_state_delay_targets(
                Branch3::new(Detection::Detect, Detection::Done, Detection::Run1),
                &game,
                &method,
                targets,
                target,
                shiny_threshold,
            ),
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
                3000..9200,
            ),
        ];

        builder.add_states(states);
        builder
    }

    pub fn frlg_random(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        // TODO detection state builder
        let (detect, shiny_threshold) = if builder.target() == 37 || builder.target() == 27 {
            // Route 8 vulpix/sandshrew
            (
                Processing::Sprite(Game::FireRedLeafGreen, vec![16, 52, 37, 27], false),
                Duration::from_millis(2700),
            )
        } else if builder.target() == 10 {
            // Viridian forest caterpie
            (
                Processing::Sprite(Game::FireRedLeafGreen, vec![10, 11, 13, 14, 25], false),
                Duration::from_millis(2700),
            )
        } else {
            // TODO hardcoded for Route 1
            (
                Processing::Sprite(Game::FireRedLeafGreen, vec![16, 19], false),
                Duration::from_millis(3250),
            )
        };
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
                3000..9200,
            ),
        ];

        builder.add_states(states);
        builder
    }

    pub fn gen7_gift(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let game = builder.game();
        let method = builder.method();

        // Detect sprite
        // Mash B for 16 seconds
        // Detect shiny star
        // Back to start
        let states = vec![
            StateDescription::simple_sprite_state_3ds(
                Branch3::new(
                    StickyState::DetectSprite,
                    StickyState::Done,
                    StickyState::StartMashB,
                ),
                game,
                method,
                builder.target(),
                builder.target(),
            ),
            StateDescription::start_timer_state(StickyState::StartMashB, StickyState::MashB),
            StateDescription::linear_state(
                StickyState::MashB,
                vec![HuntStateOutput::button(Button::B)],
                500..500,
            ),
            StateDescription::branch_delay_state(
                Branch3::new(
                    StickyState::CheckTimer,
                    StickyState::OpenMenu,
                    StickyState::MashB,
                ),
                18000,
            ),
            StateDescription::linear_state(
                StickyState::OpenMenu,
                vec![HuntStateOutput::button(Button::X)],
                2000..2000,
            ),
            StateDescription::linear_state(
                StickyState::ToParty,
                vec![HuntStateOutput::button(Button::A)],
                2000..2000,
            ),
            StateDescription::linear_state(
                StickyState::Select,
                vec![HuntStateOutput::button(Button::A)],
                2000..2000,
            ),
            StateDescription::linear_state(
                StickyState::OpenSummary,
                vec![HuntStateOutput::button(Button::A)],
                3000..3000,
            ),
            StateDescription::linear_state(
                StickyState::ToLast,
                vec![HuntStateOutput::button(Button::Up)],
                2000..2000,
            ),
            StateDescription::simple_process_state_no_output3(
                Branch3::new(
                    StickyState::DetectStar,
                    StickyState::FoundTarget,
                    StickyState::StopHeartbeat,
                ),
                Processing::USUM_SHINY_STAR,
            ),
            StateDescription::found_target_state(StickyState::FoundTarget, StickyState::Done),
            StateDescription::deadend_state(StickyState::Done),
            StateDescription::clear_atomic_state(
                StickyState::StopHeartbeat,
                StickyState::NextAttempt,
            ),
            StateDescription::linear_state(StickyState::NextAttempt, vec![], 500..2000),
        ];

        builder.add_states(states);
        builder
    }

    pub fn gen7_random_encounter(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let timer = if builder.target() == 806 {
            16300
        } else {
            10400
        };

        if builder.target() == 806 {
            let states = vec![
                StateDescription::simple_process_state_no_output(
                    Branch2::new(Detection::PrePreEnterEncounter, Detection::WaitPrePreEnter),
                    Processing::USUMBottomScreenInv(5.0),
                ),
                StateDescription::simple_process_state_no_output(
                    Branch2::new(Detection::WaitPrePreEnter, Detection::PreEnterEncounter),
                    Processing::USUMBottomScreen(5.0),
                ),
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
                StateDescription::branch_last_delay_state_plus_range(
                    Branch3::new(Detection::Run1, Detection::Toggle, Detection::Run2),
                    timer,
                    10500..15000,
                ),
                StateDescription::found_target_state(Detection::Toggle, Detection::Done),
                StateDescription::deadend_state(Detection::Done),
                StateDescription::incr_encounter_state(Detection::Run2, Detection::Run3),
                StateDescription::linear_state(Detection::Run3, vec![], 1000..1000),
                StateDescription::linear_state(
                    Detection::Run4,
                    vec![HuntStateOutput::button(Button::Touch(157, 224))],
                    8000..9000,
                ),
            ];

            builder.add_states(states);
        } else {
            let states = vec![
                StateDescription::simple_process_state_no_output(
                    Branch2::new(Detection::PrePreEnterEncounter, Detection::WaitPrePreEnter),
                    Processing::USUMBottomScreenInv(5.0),
                ),
                StateDescription::simple_process_state_no_output(
                    Branch2::new(Detection::WaitPrePreEnter, Detection::PreEnterEncounter),
                    Processing::USUMBottomScreen(5.0),
                ),
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
                StateDescription::linear_state(Detection::Run3, vec![], 1000..1000),
                StateDescription::linear_state(
                    Detection::Run4,
                    vec![HuntStateOutput::button(Button::Touch(157, 224))],
                    8000..9000,
                ),
            ];

            builder.add_states(states);
        }
        builder
    }
}
