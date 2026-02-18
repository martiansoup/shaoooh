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
    Clear1,
    Clear2,
    Clear3,
    AllowHeartbeat,
    WaitHeartbeat,
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
enum USUMRandom {
    CheckCounter,
    StopHeartbeat,
    SetCounter,
    Wait,
    SoftReset,
    Clear1,
    Clear2,
    Clear3,
    AllowHeartbeat,
    WaitHeartbeat,
    Title1,
    Title2,
    Title3,
    Title4,
    XToMenu,
    Down,
    AToBag,
    Left1,
    Left2,
    Left3,
    Jump,
    AToHoney,
    AToUse,
    XToMenu2,
    AToBag2,
    DecrCounter,
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

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum DelayState {
    Delay,
}

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum LoopState {
    ResetCounter,
    PressA,
    PressARetry,
    DecrCounter,
    CheckCounter,
    CheckToggle,
    Wait,
    PressB,
    Toggle,
    ToggleBack,
    PressA2,
    Done,
}

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum UtilityState {
    Init,
    StartTimer,
    UpdateTimer,
    CheckTimer,
    Up,
    Down,
    Left,
    Right,
    Entering,
    DownTo,
    Wait,
    UpToBattle,
    AToBattle,
    Down1,
    Down2,
    AForHappy,
    AForBall,
    Wait2,
    AToBattle2,
    RightToPay,
    AForPay,
    Wait3,
    IncrCounter,
    CheckCounter,
    Notify,
    Done,
}

#[derive(PartialEq, Hash, Eq, AsRefStr, Clone)]
enum FishingStates {
    TryFish,
    WaitFishActive,
    ShouldPressA,
    Delay,
    PressA,
    CheckOnHook,
    CheckNoNibble,
    NoNibble,
    ToStart,
    StartEncounter,
    WaitEncounterReady,
    ToDetect,
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
        } else if *game == Game::RubySapphire && *method == Method::RandomEncounter {
            Some(Self::rs_random(builder))
        } else if *game == Game::FireRedLeafGreen && *method == Method::SoftResetEncounter {
            Self::gen3_softreset(builder)
        } else if (*game == Game::RubySapphire || *game == Game::FireRedLeafGreen)
            && *method == Method::SoftResetGift
        {
            Self::gen3_softreset_gift(builder)
        } else if (*game == Game::HeartGoldSoulSilver && *method == Method::SoftResetGift)
            || (*game == Game::DiamondPearl && *method == Method::SoftResetEncounter)
        {
            Self::gen4_softreset(builder)
        } else if *game == Game::UltraSunUltraMoon && *method == Method::SoftResetEncounter {
            Self::gen7_softreset(builder)
        } else if *game == Game::UltraSunUltraMoon && *method == Method::SoftResetGift {
            Self::gen7_softreset_gift(builder)
        } else if *game == Game::UltraSunUltraMoon && *method == Method::RandomEncounter {
            Self::gen7_random_encounter(builder)
        } else if *game == Game::HeartGoldSoulSilver
            && *method == Method::SoftResetEncounter
            && builder.target() == 206
        {
            Some(Self::hgss_darkcave(builder))
        } else if *method == Method::Utility {
            Self::utility(builder)
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
        let states = if builder.game() == &Game::FireRedLeafGreen {
            vec![
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
            ]
        } else {
            vec![
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
            ]
        };

        builder.add_states(states);

        if builder.game() == &Game::FireRedLeafGreen {
            let delay_state = vec![StateDescription::linear_state(
                DelayState::Delay,
                vec![],
                5000..15000,
            )];

            builder.add_states(delay_state);
        }

        if builder.target() == 374
            || builder.target() == 138
            || builder.target() == 140
            || builder.target() == 142
        {
            // Beldum/Fossil Sequence
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
        } else if builder.target() == 131 {
            // Lapras
            let states2 = vec![
                StateDescription::set_counter_state(LoopState::ResetCounter, LoopState::PressA, 7),
                StateDescription::linear_state(
                    LoopState::PressA,
                    vec![HuntStateOutput::button(Button::A)],
                    2000..2500,
                ),
                StateDescription::decr_counter_state(
                    LoopState::DecrCounter,
                    LoopState::CheckCounter,
                ),
                StateDescription::choose_counter_state(
                    LoopState::CheckCounter,
                    LoopState::Wait,
                    LoopState::PressA,
                ),
                StateDescription::linear_state(LoopState::Wait, vec![], 2000..2500),
                StateDescription::choose_toggle_state(
                    LoopState::CheckToggle,
                    LoopState::ToggleBack,
                    LoopState::PressB,
                ),
                StateDescription::linear_state(
                    LoopState::PressB,
                    vec![HuntStateOutput::button(Button::B)],
                    1000..1500,
                ),
                StateDescription::toggle_state(LoopState::Toggle, LoopState::ResetCounter),
                StateDescription::toggle_state(LoopState::ToggleBack, LoopState::Done),
                StateDescription::linear_state_no_delay(LoopState::Done, vec![]),
            ];

            builder.add_states(states2);
        } else if builder.target() == 133 {
            // Eevee
            let states2 = vec![
                StateDescription::linear_state(
                    LoopState::PressA,
                    vec![HuntStateOutput::button(Button::A)],
                    2000..2500,
                ),
                StateDescription::linear_state(
                    LoopState::PressB,
                    vec![HuntStateOutput::button(Button::B)],
                    2000..2500,
                ),
            ];

            builder.add_states(states2);
        } else if builder.target() == 106 || builder.target() == 107 {
            // Hitmon(s)
            let states2 = vec![
                StateDescription::linear_state(
                    LoopState::PressA,
                    vec![HuntStateOutput::button(Button::A)],
                    2000..2500,
                ),
                StateDescription::linear_state(
                    LoopState::PressA2,
                    vec![HuntStateOutput::button(Button::A)],
                    2500..3000,
                ),
                StateDescription::linear_state(
                    LoopState::PressB,
                    vec![HuntStateOutput::button(Button::B)],
                    2000..2500,
                ),
            ];

            builder.add_states(states2);
        } else {
            return None;
        }

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
            StateDescription::linear_state(SoftResetProcess::SoftReset, sr_buttons, 3750..4250),
            StateDescription::linear_state(
                SoftResetProcess::Title1,
                vec![HuntStateOutput::button(Button::A)],
                5000..5500,
            ),
            StateDescription::linear_state(
                SoftResetProcess::Title2,
                vec![HuntStateOutput::button(Button::A)],
                3750..4250,
            ),
            StateDescription::linear_state(
                SoftResetProcess::Title3,
                vec![HuntStateOutput::button(Button::A)],
                2500..3000,
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
        } else if species >= 144 && species <= 146 || species == 150 {
            // Articuno/Zapdos/Moltres/Mewtwo

            let delay = if species == 150 {
                2000..2000
            } else {
                1500..1500
            };
            let states2 = vec![
                // Extra delay to try to improve randomness space
                StateDescription::linear_state(StartSoftResetEncounter::Delay, vec![], 2500..15000),
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press1,
                    vec![HuntStateOutput::button(Button::A)],
                    delay,
                ),
                StateDescription::linear_state(
                    StartSoftResetEncounter::Press2,
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
                // Extra delay to try to improve randomness space
                StateDescription::linear_state(StartSoftResetEncounter::Delay, vec![], 5000..25000),
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
            442 => {
                // Spiritomb
                let states2 = vec![
                    StateDescription::linear_state(
                        StartSoftResetEncounter::SkipMemory,
                        vec![HuntStateOutput::button(Button::Start)],
                        1500..2000,
                    ),
                    StateDescription::linear_state(
                        StartSoftResetEncounter::Delay,
                        vec![HuntStateOutput::button(Button::B)],
                        1500..2000,
                    ),
                    StateDescription::simple_process_state(
                        Branch3::new(
                            StartSoftResetEncounter::Press1,
                            StartSoftResetEncounter::IsEntering,
                            StartSoftResetEncounter::Press2,
                        ),
                        Processing::DP_START_ENCOUNTER,
                        HuntStateOutput::button(Button::A),
                        100..100,
                    ),
                    StateDescription::simple_process_state(
                        Branch3::new(
                            StartSoftResetEncounter::Press2,
                            StartSoftResetEncounter::IsEntering,
                            StartSoftResetEncounter::Press1,
                        ),
                        Processing::DP_START_ENCOUNTER,
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
            442 | 486 | 487 => {
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

    pub fn rs_fishing(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let states = vec![
            StateDescription::linear_state(
                FishingStates::TryFish,
                vec![HuntStateOutput::button(Button::Select)],
                100..100,
            ),
            StateDescription::simple_process_state_no_output(
                Branch2::new(FishingStates::WaitFishActive, FishingStates::ShouldPressA),
                Processing::RS_FISHING_ACTIVE,
            ),
            StateDescription::simple_process_state_no_output3(
                Branch3::new(
                    FishingStates::ShouldPressA,
                    FishingStates::PressA,
                    FishingStates::Delay,
                ),
                Processing::RS_FISHING_BITE,
            ),
            StateDescription::linear_state(
                FishingStates::PressA,
                vec![HuntStateOutput::button(Button::A)],
                500..500,
            ),
            StateDescription::linear_state(FishingStates::Delay, vec![], 50..50),
            StateDescription::simple_process_state_no_output3(
                Branch3::new(
                    FishingStates::CheckOnHook,
                    FishingStates::StartEncounter,
                    FishingStates::CheckNoNibble,
                ),
                Processing::RS_FISHING_ON_HOOK,
            ),
            StateDescription::linear_state(
                FishingStates::NoNibble,
                vec![HuntStateOutput::button(Button::A)],
                2500..3500,
            ),
            StateDescription::branch_state(FishingStates::ToStart, FishingStates::TryFish, 50..50),
            StateDescription::simple_process_state_no_output3(
                Branch3::new(
                    FishingStates::CheckNoNibble,
                    FishingStates::NoNibble,
                    FishingStates::ShouldPressA,
                ),
                Processing::RS_FISHING_NO_NIBBLE,
            ),
            StateDescription::linear_state(
                FishingStates::StartEncounter,
                vec![HuntStateOutput::button(Button::A)],
                100..100,
            ),
            StateDescription::simple_process_state_no_output(
                Branch2::new(FishingStates::WaitEncounterReady, FishingStates::ToDetect),
                Processing::FRLG_START_ENCOUNTER,
            ),
            StateDescription::linear_state(FishingStates::ToDetect, vec![], 50..50),
        ];

        builder.add_states(states);

        builder
    }

    pub fn rs_random(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        if builder.target() == 320 {
            // Wailmer
            return Self::rs_fishing(builder);
        }

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
        let states_honey = vec![
            StateDescription::linear_state(USUMRandom::Wait, vec![], 5000..12500),
            StateDescription::linear_state(
                USUMRandom::XToMenu,
                vec![HuntStateOutput::button(Button::X)],
                1000..1500,
            ),
            StateDescription::linear_state(
                USUMRandom::Down,
                vec![HuntStateOutput::button(Button::Down)],
                500..1000,
            ),
            StateDescription::linear_state(
                USUMRandom::AToBag,
                vec![HuntStateOutput::button(Button::A)],
                1500..2000,
            ),
            StateDescription::linear_state(
                USUMRandom::Left1,
                vec![HuntStateOutput::button(Button::Left)],
                500..1000,
            ),
            StateDescription::linear_state(
                USUMRandom::Left2,
                vec![HuntStateOutput::button(Button::Left)],
                500..1000,
            ),
            StateDescription::linear_state(
                USUMRandom::Left3,
                vec![HuntStateOutput::button(Button::Left)],
                500..1000,
            ),
            StateDescription::linear_state(
                USUMRandom::AToHoney,
                vec![HuntStateOutput::button(Button::A)],
                500..1000,
            ),
            StateDescription::linear_state(
                USUMRandom::AToUse,
                vec![HuntStateOutput::button(Button::A)],
                500..500,
            ),
        ];
        builder.add_states(states);
        match target {
            797 | 799 => builder.add_states(states_press),
            806 => builder.add_states(states_honey),
            _ => builder.add_states(states_walk),
        }

        Some(builder)
    }

    pub fn gen7_softreset_gift(mut builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let sr_buttons = vec![
            HuntStateOutput::new(Button::L, Delay::Half),
            HuntStateOutput::new(Button::R, Delay::Half),
            HuntStateOutput::new(Button::Start, Delay::Half),
            HuntStateOutput::new(Button::Select, Delay::Half),
        ];
        let states = vec![
            StateDescription::linear_state(USUM::SoftReset1, sr_buttons, 50..50),
            //StateDescription::linear_state(USUM::SoftReset2, sr_buttons, 50..50),
            // Add states to clear button presses in case packet got missed
            StateDescription::linear_state(
                USUM::Clear1,
                vec![HuntStateOutput::button(Button::Up)],
                1000..2000,
            ),
            StateDescription::linear_state(
                USUM::Clear2,
                vec![HuntStateOutput::button(Button::Up)],
                1000..2000,
            ),
            StateDescription::linear_state(
                USUM::Clear3,
                vec![HuntStateOutput::button(Button::Up)],
                9000..8000,
            ),
            StateDescription::set_atomic_state(USUM::AllowHeartbeat, USUM::WaitHeartbeat),
            StateDescription::linear_state(USUM::WaitHeartbeat, vec![], 2500..2500),
            StateDescription::linear_state(
                USUM::Title1,
                vec![HuntStateOutput::button(Button::A)],
                50..50,
            ),
            StateDescription::linear_state(
                USUM::Title2,
                vec![HuntStateOutput::button(Button::A)],
                2000..2250,
            ),
            StateDescription::linear_state(
                USUM::Title3,
                vec![HuntStateOutput::button(Button::A)],
                50..50,
            ),
            StateDescription::linear_state(
                USUM::Title4,
                vec![HuntStateOutput::button(Button::A)],
                2500..12750,
            ),
        ];
        builder.add_states(states);

        if builder.target() == 803 {
            // Poipole
            let states_get = vec![
                StateDescription::set_counter_state(LoopState::ResetCounter, LoopState::PressA, 5),
                StateDescription::linear_state(
                    LoopState::PressA,
                    vec![HuntStateOutput::button(Button::A)],
                    50..50,
                ),
                StateDescription::linear_state(
                    LoopState::PressARetry,
                    vec![HuntStateOutput::button(Button::A)],
                    2500..2500,
                ),
                StateDescription::decr_counter_state(
                    LoopState::DecrCounter,
                    LoopState::CheckCounter,
                ),
                StateDescription::choose_counter_state(
                    LoopState::CheckCounter,
                    LoopState::Wait,
                    LoopState::PressA,
                ),
                StateDescription::linear_state(LoopState::Wait, vec![], 5000..5000),
                StateDescription::linear_state(
                    LoopState::PressA2,
                    vec![HuntStateOutput::button(Button::A)],
                    3000..3000,
                ),
                StateDescription::linear_state_no_delay(LoopState::Done, vec![]),
            ];

            builder.add_states(states_get);
        } else {
            return None;
        }

        Some(builder)
    }

    pub fn gen7_random_encounter(mut builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let sr_buttons = vec![
            HuntStateOutput::new(Button::L, Delay::Half),
            HuntStateOutput::new(Button::R, Delay::Half),
            HuntStateOutput::new(Button::Start, Delay::Half),
            HuntStateOutput::new(Button::Select, Delay::Half),
        ];

        let states = vec![
            StateDescription::choose_counter_state(
                USUMRandom::CheckCounter,
                USUMRandom::StopHeartbeat,
                USUMRandom::XToMenu2,
            ),
            StateDescription::clear_atomic_state(USUMRandom::StopHeartbeat, USUMRandom::SetCounter),
            StateDescription::set_counter_state(USUMRandom::SetCounter, USUMRandom::Wait, 64),
            StateDescription::linear_state(USUMRandom::Wait, vec![], 500..1500),
            // Do soft reset
            StateDescription::linear_state(USUMRandom::SoftReset, sr_buttons, 50..50),
            // Add states to clear button presses in case packet got missed
            StateDescription::linear_state(
                USUMRandom::Clear1,
                vec![HuntStateOutput::button(Button::Up)],
                1000..2000,
            ),
            StateDescription::linear_state(
                USUMRandom::Clear2,
                vec![HuntStateOutput::button(Button::Up)],
                1000..2000,
            ),
            StateDescription::linear_state(
                USUMRandom::Clear3,
                vec![HuntStateOutput::button(Button::Up)],
                9000..8000,
            ),
            StateDescription::set_atomic_state(
                USUMRandom::AllowHeartbeat,
                USUMRandom::WaitHeartbeat,
            ),
            StateDescription::linear_state(USUMRandom::WaitHeartbeat, vec![], 2500..2500),
            StateDescription::linear_state(
                USUMRandom::Title1,
                vec![HuntStateOutput::button(Button::A)],
                50..50,
            ),
            StateDescription::linear_state(
                USUMRandom::Title2,
                vec![HuntStateOutput::button(Button::A)],
                2000..2250,
            ),
            StateDescription::linear_state(
                USUMRandom::Title3,
                vec![HuntStateOutput::button(Button::A)],
                50..50,
            ),
            StateDescription::linear_state(
                USUMRandom::Title4,
                vec![HuntStateOutput::button(Button::A)],
                5500..12750,
            ),
            // Go to bag after soft reset
            StateDescription::linear_state(
                USUMRandom::XToMenu,
                vec![HuntStateOutput::button(Button::X)],
                1000..1500,
            ),
            StateDescription::linear_state(
                USUMRandom::Down,
                vec![HuntStateOutput::button(Button::Down)],
                500..1000,
            ),
            StateDescription::linear_state(
                USUMRandom::AToBag,
                vec![HuntStateOutput::button(Button::A)],
                1500..2000,
            ),
            StateDescription::linear_state(
                USUMRandom::Left1,
                vec![HuntStateOutput::button(Button::Left)],
                500..1000,
            ),
            StateDescription::linear_state(
                USUMRandom::Left2,
                vec![HuntStateOutput::button(Button::Left)],
                500..1000,
            ),
            StateDescription::linear_state(
                USUMRandom::Left3,
                vec![HuntStateOutput::button(Button::Left)],
                500..1000,
            ),
            StateDescription::branch_state(USUMRandom::Jump, USUMRandom::AToHoney, 50..50),
            // Go to bag after running
            StateDescription::linear_state(
                USUMRandom::XToMenu2,
                vec![HuntStateOutput::button(Button::X)],
                1000..1500,
            ),
            StateDescription::linear_state(
                USUMRandom::AToBag2,
                vec![HuntStateOutput::button(Button::A)],
                1500..2000,
            ),
            StateDescription::linear_state(
                USUMRandom::AToHoney,
                vec![HuntStateOutput::button(Button::A)],
                500..1000,
            ),
            StateDescription::decr_counter_state(USUMRandom::DecrCounter, USUMRandom::AToUse),
            StateDescription::linear_state(
                USUMRandom::AToUse,
                vec![HuntStateOutput::button(Button::A)],
                500..500,
            ),
        ];

        builder.add_states(states);
        // If counter non-zero
        // Go to bag, press honey
        //   X, A, A
        // If counter zero
        // Soft reset
        // Set counter 64
        // Go to bag, press honey
        Some(builder)
    }

    pub fn utility(mut builder: HuntFSMBuilder) -> Option<HuntFSMBuilder> {
        let target = builder.target();
        let g7_move = 750;

        let states = if target == 1 {
            // Get money in ORAS using HappyHour/PayDay
            Some(vec![
                StateDescription::linear_state(
                    UtilityState::Left,
                    vec![HuntStateOutput::button(Button::Left)],
                    g7_move..g7_move,
                ),
                StateDescription::start_timer_state(UtilityState::StartTimer, UtilityState::Up),
                StateDescription::linear_state(
                    UtilityState::Up,
                    vec![HuntStateOutput::button(Button::Up)],
                    g7_move..g7_move,
                ),
                StateDescription::linear_state(
                    UtilityState::Down,
                    vec![HuntStateOutput::button(Button::Down)],
                    g7_move..g7_move,
                ),
                StateDescription::update_timer_state(
                    UtilityState::UpdateTimer,
                    UtilityState::CheckTimer,
                ),
                StateDescription::branch_last_delay_state(
                    Branch3::new(
                        UtilityState::CheckTimer,
                        UtilityState::Wait,
                        UtilityState::Up,
                    ),
                    8000,
                ),
                StateDescription::linear_state(UtilityState::Wait, vec![], 10000..10000),
                StateDescription::linear_state(
                    UtilityState::UpToBattle,
                    vec![HuntStateOutput::button(Button::Up)],
                    1000..1000,
                ),
                StateDescription::linear_state(
                    UtilityState::AToBattle,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1000,
                ),
                StateDescription::linear_state(
                    UtilityState::AForHappy,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1000,
                ),
                StateDescription::linear_state(UtilityState::Wait2, vec![], 15000..15000),
                StateDescription::linear_state(
                    UtilityState::AToBattle2,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1000,
                ),
                StateDescription::linear_state(
                    UtilityState::RightToPay,
                    vec![HuntStateOutput::button(Button::Right)],
                    1000..1000,
                ),
                StateDescription::linear_state(
                    UtilityState::AForPay,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1000,
                ),
                StateDescription::linear_state(
                    UtilityState::Wait3,
                    vec![HuntStateOutput::button(Button::A)],
                    15000..15000,
                ),
                StateDescription::incr_counter_state(
                    UtilityState::IncrCounter,
                    UtilityState::CheckCounter,
                ),
                StateDescription::choose_counter_state_val(
                    UtilityState::CheckCounter,
                    UtilityState::Notify,
                    UtilityState::Left,
                    20,
                ),
                StateDescription::found_target_state(UtilityState::Notify, UtilityState::Done),
                StateDescription::deadend_state(UtilityState::Done),
            ])
        } else if target == 2 {
            // Catch (when guaranteed)
            Some(vec![
                StateDescription::linear_state(
                    UtilityState::Left,
                    vec![HuntStateOutput::button(Button::Left)],
                    g7_move..g7_move,
                ),
                //StateDescription::start_timer_state(UtilityState::StartTimer, UtilityState::Up),
                StateDescription::simple_process_state(
                    Branch3::new(UtilityState::Up, UtilityState::Entering, UtilityState::Down),
                    Processing::USUMBottomScreenInv(5.0),
                    HuntStateOutput::button(Button::Up),
                    g7_move..g7_move,
                ),
                StateDescription::simple_process_state(
                    Branch3::new(UtilityState::Down, UtilityState::Entering, UtilityState::Up),
                    Processing::USUMBottomScreenInv(5.0),
                    HuntStateOutput::button(Button::Down),
                    g7_move..g7_move,
                ),
                //StateDescription::linear_state(UtilityState::Up, vec![HuntStateOutput::button(Button::Up)], g7_move..g7_move),
                //StateDescription::linear_state(UtilityState::Down, vec![HuntStateOutput::button(Button::Down)], g7_move..g7_move),
                //StateDescription::update_timer_state(UtilityState::UpdateTimer, UtilityState::CheckTimer),
                //StateDescription::branch_last_delay_state(Branch3::new(UtilityState::CheckTimer, UtilityState::Wait, UtilityState::Up), 8000),
                StateDescription::simple_process_state_no_output(
                    Branch2::new(UtilityState::Entering, UtilityState::Wait),
                    Processing::USUMBottomScreen(5.0),
                ),
                StateDescription::linear_state(UtilityState::Wait, vec![], 1000..1000),
                // DownToBag
                StateDescription::linear_state(
                    UtilityState::UpToBattle,
                    vec![HuntStateOutput::button(Button::Down)],
                    1000..1000,
                ),
                // AToBag
                StateDescription::linear_state(
                    UtilityState::AToBattle,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1000,
                ),
                StateDescription::linear_state(
                    UtilityState::Down1,
                    vec![HuntStateOutput::button(Button::Down)],
                    1000..1000,
                ),
                StateDescription::linear_state(
                    UtilityState::Down2,
                    vec![HuntStateOutput::button(Button::Down)],
                    1000..1000,
                ),
                // AToUseBall
                StateDescription::linear_state(
                    UtilityState::AForHappy,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1000,
                ),
                StateDescription::linear_state(
                    UtilityState::AForBall,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1000,
                ),
                StateDescription::linear_state(UtilityState::Wait2, vec![], 22000..22000),
                // B to not nickname
                StateDescription::linear_state(
                    UtilityState::AToBattle2,
                    vec![HuntStateOutput::button(Button::B)],
                    3000..3000,
                ),
                // B to done
                StateDescription::linear_state(
                    UtilityState::RightToPay,
                    vec![HuntStateOutput::button(Button::B)],
                    3000..3000,
                ),
                StateDescription::incr_encounter_state(UtilityState::AForPay, UtilityState::Wait3),
                StateDescription::linear_state(
                    UtilityState::Wait3,
                    vec![HuntStateOutput::button(Button::A)],
                    1000..1000,
                ),
            ])
        } else {
            None
        };

        match states {
            Some(s) => {
                builder.add_states(s);
                Some(builder)
            }
            None => None,
        }
    }
}
