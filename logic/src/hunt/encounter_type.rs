use strum_macros::AsRefStr;

use crate::{
    app::{Game, Method},
    control::{Button, Delay},
    hunt::{Branch2, Branch3, HuntFSMBuilder, HuntStateOutput, StateDescription},
    vision::Processing,
};

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum TryGetEncounter {
    Init,
    Up,
    Down,
    Left,
    Right,
    Entering,
}

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum SoftResetProcess {
    SoftReset,
    Title1,
    Title2,
    Title3,
    Title4,
    SkipMemory,
    GetGift,
}

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum StartSoftResetEncounter {
    SkipMemory,
    Delay,
    Press1,
    Press2,
    Press3,
    Press4,
    Press5,
    Press6,
    Press7,
    IsEntering,
    Entering,
}

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum USUM {
    SoftReset1,
    SoftReset2,
    AllowHeartbeat,
    Title1,
    Title2,
    Title3,
    Title4,
    Circle1,
    Circle2,
    Circle3,
    Circle4,
}

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum DarkCave {
    Start,
    CheckToggle,
    First,
    Second,
    SoftReset,
    Title1,
    Title2,
    Title3,
    Title4,
    SetToggle,
    SetToggle2,
    Down1,
    Down2,
    Right,
    Smash,
    Smash2,
    Smash3,
    B1,
    B2,
    StartTimer,
    CheckEncounter,
    BranchEncounter,
    Entering,
}

pub struct EncounterTypeResolver {}

impl EncounterTypeResolver {
    const MOVE_DELAY: u64 = 75;

    pub fn add_states(builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let game = builder.game();
        let method = builder.method();
        log::info!("Adding Encounter Type states for '{:?}/{:?}'", game, method);

        if *game == Game::FireRedLeafGreen && *method == Method::RandomEncounter {
            Some(Self::frlg_random(builder))
        } else if *game == Game::FireRedLeafGreen && *method == Method::SoftResetEncounter {
            Self::gen3_softreset(builder)
        } else if *game == Game::RubySapphire && *method == Method::SoftResetGift {
            Self::gen3_softreset_gift(builder)
        } else if (*game == Game::HeartGoldSoulSilver && *method == Method::SoftResetGift)
            || (*game == Game::DiamondPearl && *method == Method::SoftResetEncounter)
        {
            Self::gen4_softreset(builder)
        } else if *game == Game::UltraSunUltraMoon && *method == Method::SoftResetEncounter {
            Self::gen7_softreset(builder)
        } else if *game == Game::HeartGoldSoulSilver
            && *method == Method::SoftResetEncounter
            && builder.target() == 206
        {
            Some(Self::hgss_darkcave(builder))
        } else {
            None
        }
    }

    pub fn hgss_darkcave(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let sr_buttons = vec![
            HuntStateOutput::new(Button::L, Delay::Tenth),
            HuntStateOutput::new(Button::R, Delay::Tenth),
            HuntStateOutput::new(Button::Start, Delay::Tenth),
            HuntStateOutput::new(Button::Select, Delay::Tenth),
        ];

        let states = vec![
            StateDescription::linear_state_no_delay(DarkCave::Start, vec![]),
            StateDescription::choose_toggle_state(
                DarkCave::CheckToggle,
                DarkCave::Second,
                DarkCave::First,
            ),
            StateDescription::linear_state_no_delay(DarkCave::First, vec![]),
            StateDescription::linear_state(DarkCave::SoftReset, sr_buttons, 7500..8000),
            StateDescription::linear_state(
                DarkCave::Title1,
                vec![HuntStateOutput::button(Button::A)],
                4000..4250,
            ),
            StateDescription::linear_state(
                DarkCave::Title2,
                vec![HuntStateOutput::button(Button::A)],
                4500..4750,
            ),
            StateDescription::linear_state(
                DarkCave::Title3,
                vec![HuntStateOutput::button(Button::A)],
                3500..3750,
            ),
            StateDescription::linear_state(
                DarkCave::Title4,
                vec![HuntStateOutput::button(Button::A)],
                3000..3250,
            ),
            StateDescription::toggle_state(DarkCave::SetToggle, DarkCave::Smash),
            StateDescription::linear_state_no_delay(DarkCave::Second, vec![]),
            StateDescription::linear_state(
                DarkCave::B1,
                vec![HuntStateOutput::new(Button::B, Delay::Tenth)],
                2500..2500,
            ),
            StateDescription::linear_state(
                DarkCave::B2,
                vec![HuntStateOutput::new(Button::B, Delay::Tenth)],
                2500..2500,
            ),
            StateDescription::linear_state(
                DarkCave::Down1,
                vec![HuntStateOutput::new(Button::Down, Delay::Tenth)],
                250..250,
            ),
            StateDescription::linear_state(
                DarkCave::Down2,
                vec![HuntStateOutput::new(Button::Down, Delay::Tenth)],
                250..250,
            ),
            StateDescription::linear_state(
                DarkCave::Right,
                vec![HuntStateOutput::new(Button::Right, Delay::Tenth)],
                250..250,
            ),
            StateDescription::toggle_state(DarkCave::SetToggle2, DarkCave::Smash),
            StateDescription::linear_state(
                DarkCave::Smash,
                vec![HuntStateOutput::new(Button::A, Delay::Tenth)],
                2000..2000,
            ),
            StateDescription::linear_state(
                DarkCave::Smash2,
                vec![HuntStateOutput::new(Button::A, Delay::Tenth)],
                2000..2000,
            ),
            StateDescription::linear_state(
                DarkCave::Smash3,
                vec![HuntStateOutput::new(Button::A, Delay::Tenth)],
                2000..2000,
            ),
            StateDescription::start_timer_state(DarkCave::StartTimer, DarkCave::CheckEncounter),
            StateDescription::simple_process_state_no_output3(
                Branch3::new(
                    DarkCave::CheckEncounter,
                    DarkCave::Entering,
                    DarkCave::BranchEncounter,
                ),
                Processing::HGSS_BLACK_SCREEN,
            ),
            StateDescription::branch_delay_state(
                Branch3::new(
                    DarkCave::BranchEncounter,
                    DarkCave::Start,
                    DarkCave::CheckEncounter,
                ),
                8000,
            ),
            StateDescription::linear_state_no_delay(DarkCave::Entering, vec![]),
        ];

        builder.add_states(states);
        builder
    }

    pub fn gen3_softreset_gift(mut builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let sr_buttons = vec![
            HuntStateOutput::new(Button::A, Delay::Tenth),
            HuntStateOutput::new(Button::B, Delay::Tenth),
            HuntStateOutput::new(Button::Start, Delay::Tenth),
            HuntStateOutput::new(Button::Select, Delay::Tenth),
        ];
        let states = vec![
            StateDescription::linear_state(SoftResetProcess::SoftReset, sr_buttons, 3750..3750),
            StateDescription::linear_state(
                SoftResetProcess::Title1,
                vec![HuntStateOutput::button(Button::A)],
                5000..5000,
            ),
            StateDescription::linear_state(
                SoftResetProcess::Title2,
                vec![HuntStateOutput::button(Button::A)],
                3750..3750,
            ),
            StateDescription::linear_state(
                SoftResetProcess::Title3,
                vec![HuntStateOutput::button(Button::A)],
                2500..2500,
            ),
            StateDescription::linear_state(
                SoftResetProcess::Title4,
                vec![HuntStateOutput::button(Button::A)],
                2000..2500,
            ),
        ];

        builder.add_states(states);

        if builder.target() != 374 {
            return None;
        }

        // Beldum Sequence
        let states2 = vec![
            StateDescription::linear_state(
                StartSoftResetEncounter::Press1,
                vec![HuntStateOutput::button(Button::A)],
                1000..5500,
            ),
            StateDescription::linear_state(
                StartSoftResetEncounter::Press2,
                vec![HuntStateOutput::button(Button::A)],
                750..1250,
            ),
            StateDescription::linear_state(
                StartSoftResetEncounter::Press3,
                vec![HuntStateOutput::button(Button::A)],
                1000..1500,
            ),
            StateDescription::linear_state(
                StartSoftResetEncounter::Press4,
                vec![HuntStateOutput::button(Button::A)],
                5000..5000,
            ),
            StateDescription::linear_state(
                StartSoftResetEncounter::Press5,
                vec![HuntStateOutput::button(Button::B)],
                1000..1000,
            ),
        ];

        builder.add_states(states2);

        Some(builder)
    }

    pub fn gen3_softreset(mut builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let sr_buttons = vec![
            HuntStateOutput::new(Button::A, Delay::Tenth),
            HuntStateOutput::new(Button::B, Delay::Tenth),
            HuntStateOutput::new(Button::Start, Delay::Tenth),
            HuntStateOutput::new(Button::Select, Delay::Tenth),
        ];
        let states = vec![
            StateDescription::linear_state(SoftResetProcess::SoftReset, sr_buttons, 3750..3750),
            StateDescription::linear_state(
                SoftResetProcess::Title1,
                vec![HuntStateOutput::button(Button::A)],
                5000..5000,
            ),
            StateDescription::linear_state(
                SoftResetProcess::Title2,
                vec![HuntStateOutput::button(Button::A)],
                3750..3750,
            ),
            StateDescription::linear_state(
                SoftResetProcess::Title3,
                vec![HuntStateOutput::button(Button::A)],
                2500..2500,
            ),
            StateDescription::linear_state(
                SoftResetProcess::SkipMemory,
                vec![HuntStateOutput::button(Button::B)],
                3000..3500,
            ),
        ];

        builder.add_states(states);

        let species = builder.target();
        if species == 143 {
            // PokeFlute Sequence for Snorlax
            let states2 = vec![
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press1,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1500,
                ),
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press2,
                    vec![HuntStateOutput::button(Button::A)],
                    9000..9250,
                ),
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press3,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1500,
                ),
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press4,
                    vec![HuntStateOutput::button(Button::A)],
                    0..0,
                ),
                StateDescription::simple_process_state_no_output(
                    Branch2::new(
                        StartSoftResetEncounter::IsEntering,
                        StartSoftResetEncounter::Entering,
                    ),
                    Processing::FRLG_START_ENCOUNTER,
                ),
                StateDescription::linear_state_no_delay(StartSoftResetEncounter::Entering, vec![]),
            ];

            builder.add_states(states2);
        } else if species == 97 {
            // Sequence for Lostelle Hypno encounter
            let states2 = vec![
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press1,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1500,
                ),
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press2,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1500,
                ),
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press3,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1500,
                ),
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press4,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1500,
                ),
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press5,
                    vec![HuntStateOutput::button(Button::A)],
                    2000..2500,
                ),
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press6,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1500,
                ),
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press7,
                    vec![HuntStateOutput::button(Button::A)],
                    0..0,
                ),
                StateDescription::simple_process_state_no_output(
                    Branch2::new(
                        StartSoftResetEncounter::IsEntering,
                        StartSoftResetEncounter::Entering,
                    ),
                    Processing::FRLG_START_ENCOUNTER,
                ),
                StateDescription::linear_state_no_delay(StartSoftResetEncounter::Entering, vec![]),
            ];

            builder.add_states(states2);
        } else {
            return None;
        }

        Some(builder)
    }

    pub fn gen4_softreset(mut builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let sr_buttons = vec![
            HuntStateOutput::new(Button::L, Delay::Half),
            HuntStateOutput::new(Button::R, Delay::Half),
            HuntStateOutput::new(Button::Start, Delay::Half),
            HuntStateOutput::new(Button::Select, Delay::Half),
        ];
        let states = vec![
            StateDescription::linear_state(SoftResetProcess::SoftReset, sr_buttons, 7500..8000),
            StateDescription::linear_state(
                SoftResetProcess::Title1,
                vec![HuntStateOutput::button(Button::A)],
                4000..4250,
            ),
            StateDescription::linear_state(
                SoftResetProcess::Title2,
                vec![HuntStateOutput::button(Button::A)],
                2500..2750,
            ),
            StateDescription::linear_state(
                SoftResetProcess::Title3,
                vec![HuntStateOutput::button(Button::A)],
                3500..3750,
            ),
            StateDescription::linear_state(
                SoftResetProcess::Title4,
                vec![HuntStateOutput::button(Button::A)],
                3000..3250,
            ),
        ];
        builder.add_states(states);

        match builder.target() {
            152 | 155 | 158 => {
                // Starters
                let states2 = vec![StateDescription::linear_state(
                    SoftResetProcess::GetGift,
                    vec![HuntStateOutput::button(Button::A)],
                    3500..4500,
                )];
                builder.add_states(states2);
                Some(builder)
            }
            486 | 487 => {
                // Legendaries
                // Needs Setup/Decide/Decr
                // Count button press method
                // let states2 = vec![
                //     StateDescription::linear_state(StartSoftResetEncounter::Delay, vec![], 0..1000),
                //     StateDescription::set_counter_state(StartSoftResetEncounter::Setup, StartSoftResetEncounter::Press1, 8),
                //     StateDescription::linear_state(StartSoftResetEncounter::Press1, vec![HuntStateOutput::button(Button::A)], 1500..1500),
                //     StateDescription::decr_counter_state(StartSoftResetEncounter::Decr, StartSoftResetEncounter::Decide),
                //     StateDescription::choose_counter_state(StartSoftResetEncounter::Decide, StartSoftResetEncounter::Press2, StartSoftResetEncounter::Press1),
                //     // Last A with no delay for encounter start
                //     StateDescription::linear_state(StartSoftResetEncounter::Press2, vec![HuntStateOutput::button(Button::A)], 0..0),
                //     StateDescription::simple_process_state_no_output(
                //         StartSoftResetEncounter::IsEntering,
                //         StartSoftResetEncounter::Entering,
                //         Processing::DP_START_ENCOUNTER,
                //     ),
                //     StateDescription::linear_state_no_delay(
                //         StartSoftResetEncounter::Entering,
                //         vec![],
                //     ),
                // ];
                // Mash method
                let states2 = vec![
                    StateDescription::linear_state(
                        StartSoftResetEncounter::SkipMemory,
                        vec![HuntStateOutput::button(Button::Start)],
                        500..1000,
                    ),
                    StateDescription::linear_state(
                        StartSoftResetEncounter::Delay,
                        vec![HuntStateOutput::button(Button::B)],
                        500..1000,
                    ),
                    StateDescription::simple_process_state(
                        Branch3::new(
                            StartSoftResetEncounter::Press1,
                            StartSoftResetEncounter::IsEntering,
                            StartSoftResetEncounter::Press2,
                        ),
                        Processing::DP_START_ENCOUNTER_WHITE,
                        HuntStateOutput::button(Button::A),
                        100..100,
                    ),
                    StateDescription::simple_process_state(
                        Branch3::new(
                            StartSoftResetEncounter::Press2,
                            StartSoftResetEncounter::IsEntering,
                            StartSoftResetEncounter::Press1,
                        ),
                        Processing::DP_START_ENCOUNTER_WHITE,
                        HuntStateOutput::button(Button::A),
                        100..100,
                    ),
                    StateDescription::simple_process_state_no_output(
                        Branch2::new(
                            StartSoftResetEncounter::IsEntering,
                            StartSoftResetEncounter::Entering,
                        ),
                        Processing::DP_START_ENCOUNTER,
                    ),
                    StateDescription::linear_state_no_delay(
                        StartSoftResetEncounter::Entering,
                        vec![],
                    ),
                ];

                builder.add_states(states2);
                Some(builder)
            }
            _ => None,
        }
    }

    pub fn frlg_random(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let states = vec![
            StateDescription::choose_toggle_state(
                TryGetEncounter::Init,
                TryGetEncounter::Up,
                TryGetEncounter::Left,
            ),
            StateDescription::simple_process_state(
                Branch3::new(
                    TryGetEncounter::Up,
                    TryGetEncounter::Entering,
                    TryGetEncounter::Down,
                ),
                Processing::FRLG_START_ENCOUNTER,
                HuntStateOutput::new(Button::Up, Delay::Tenth),
                Self::MOVE_DELAY..Self::MOVE_DELAY,
            ),
            StateDescription::simple_process_state(
                Branch3::new(
                    TryGetEncounter::Down,
                    TryGetEncounter::Entering,
                    TryGetEncounter::Up,
                ),
                Processing::FRLG_START_ENCOUNTER,
                HuntStateOutput::new(Button::Down, Delay::Tenth),
                Self::MOVE_DELAY..Self::MOVE_DELAY,
            ),
            StateDescription::simple_process_state(
                Branch3::new(
                    TryGetEncounter::Left,
                    TryGetEncounter::Entering,
                    TryGetEncounter::Right,
                ),
                Processing::FRLG_START_ENCOUNTER,
                HuntStateOutput::new(Button::Left, Delay::Tenth),
                Self::MOVE_DELAY..Self::MOVE_DELAY,
            ),
            StateDescription::simple_process_state(
                Branch3::new(
                    TryGetEncounter::Right,
                    TryGetEncounter::Entering,
                    TryGetEncounter::Left,
                ),
                Processing::FRLG_START_ENCOUNTER,
                HuntStateOutput::new(Button::Right, Delay::Tenth),
                Self::MOVE_DELAY..Self::MOVE_DELAY,
            ),
            StateDescription::linear_state_no_delay(TryGetEncounter::Entering, vec![]),
        ];

        builder.add_states(states);
        builder
    }

    pub fn gen7_softreset(mut builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let target = builder.target();
        let sr_buttons = vec![
            HuntStateOutput::new(Button::L, Delay::Half),
            HuntStateOutput::new(Button::R, Delay::Half),
            HuntStateOutput::new(Button::Start, Delay::Half),
            HuntStateOutput::new(Button::Select, Delay::Half),
        ];
        let states = vec![
            StateDescription::linear_state(USUM::SoftReset1, sr_buttons.clone(), 50..50),
            StateDescription::linear_state(USUM::SoftReset2, sr_buttons, 11000..12000),
            StateDescription::set_atomic_state(USUM::AllowHeartbeat, USUM::Title1),
            StateDescription::linear_state(
                USUM::Title1,
                vec![HuntStateOutput::button(Button::A)],
                2000..2250,
            ),
            StateDescription::linear_state(
                USUM::Title2,
                vec![HuntStateOutput::button(Button::A)],
                2500..2750,
            ),
            StateDescription::linear_state(
                USUM::Title3,
                vec![HuntStateOutput::button(Button::A)],
                2000..2250,
            ),
            StateDescription::linear_state(
                USUM::Title4,
                vec![HuntStateOutput::button(Button::A)],
                2500..2750,
            ),
        ];
        let states_walk = vec![
            StateDescription::linear_state(
                USUM::Circle1,
                vec![HuntStateOutput::new(Button::Circle(128, 250), Delay::Half)],
                250..250,
            ),
            StateDescription::linear_state(
                USUM::Circle2,
                vec![HuntStateOutput::new(Button::Circle(128, 250), Delay::Half)],
                250..250,
            ),
            StateDescription::linear_state(
                USUM::Circle3,
                vec![HuntStateOutput::new(Button::Circle(128, 250), Delay::Half)],
                250..250,
            ),
            StateDescription::linear_state(
                USUM::Circle4,
                vec![HuntStateOutput::new(Button::Circle(128, 250), Delay::Half)],
                4000..4000,
            ),
        ];
        let states_press = vec![
            StateDescription::linear_state(
                USUM::Circle1,
                vec![HuntStateOutput::new(Button::A, Delay::Half)],
                250..250,
            ),
            StateDescription::linear_state(
                USUM::Circle2,
                vec![HuntStateOutput::new(Button::A, Delay::Half)],
                250..250,
            ),
        ];
        builder.add_states(states);
        match target {
            797 | 799 => builder.add_states(states_press),
            _ => builder.add_states(states_walk),
        }

        Some(builder)
    }
}
