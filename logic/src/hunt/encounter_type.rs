use strum_macros::AsRefStr;

use crate::{
    app::{Game, Method},
    control::{Button, Delay},
    hunt::{HuntFSMBuilder, HuntStateOutput, StateDescription},
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
    Press1,
    Press2,
    Press3,
    Press4,
    IsEntering,
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
            Some(Self::gen3_softreset(builder))
        } else if *game == Game::HeartGoldSoulSilver && *method == Method::SoftResetGift {
            Some(Self::gen4_softreset(builder))
        } else {
            None
        }
    }

    pub fn gen3_softreset(mut builder : HuntFSMBuilder) -> HuntFSMBuilder {
        let sr_buttons = vec![
            HuntStateOutput::new(Button::A, Delay::Tenth),
            HuntStateOutput::new(Button::B, Delay::Tenth),
            HuntStateOutput::new(Button::Start, Delay::Tenth),
            HuntStateOutput::new(Button::Select, Delay::Tenth)
        ];
        let states = vec![
            StateDescription::linear_state(SoftResetProcess::SoftReset, sr_buttons, 3750..3750),
            StateDescription::linear_state(SoftResetProcess::Title1, vec![HuntStateOutput::button(Button::A)], 5000..5000),
            StateDescription::linear_state(SoftResetProcess::Title2, vec![HuntStateOutput::button(Button::A)], 3750..3750),
            StateDescription::linear_state(SoftResetProcess::Title3, vec![HuntStateOutput::button(Button::A)], 2500..2500),
            StateDescription::linear_state(SoftResetProcess::SkipMemory, vec![HuntStateOutput::button(Button::B)], 3000..3500),
        ];

        builder.add_states(states);

        let states2 = vec![
            StateDescription::linear_state(StartSoftResetEncounter::Press1, vec![HuntStateOutput::button(Button::A)], 1000..1500),
            StateDescription::linear_state(StartSoftResetEncounter::Press2, vec![HuntStateOutput::button(Button::A)], 9000..9250),
            StateDescription::linear_state(StartSoftResetEncounter::Press3, vec![HuntStateOutput::button(Button::A)], 1000..1500),
            StateDescription::linear_state(StartSoftResetEncounter::Press4, vec![HuntStateOutput::button(Button::A)], 0..0),
            StateDescription::simple_process_state_no_output(
                StartSoftResetEncounter::IsEntering,
                StartSoftResetEncounter::Entering,
                Processing::FRLG_START_ENCOUNTER,
            ),
            StateDescription::linear_state_no_delay(StartSoftResetEncounter::Entering, vec![]),
        ];

        builder.add_states(states2);

        builder
    }

    pub fn gen4_softreset(mut builder : HuntFSMBuilder) -> HuntFSMBuilder {
        let sr_buttons = vec![
            HuntStateOutput::new(Button::L, Delay::Tenth),
            HuntStateOutput::new(Button::R, Delay::Tenth),
            HuntStateOutput::new(Button::Start, Delay::Tenth),
            HuntStateOutput::new(Button::Select, Delay::Tenth)
        ];
        let states = vec![
            StateDescription::linear_state(SoftResetProcess::SoftReset, sr_buttons, 7500..8000),
            StateDescription::linear_state(SoftResetProcess::Title1, vec![HuntStateOutput::button(Button::A)], 4000..4250),
            StateDescription::linear_state(SoftResetProcess::Title2, vec![HuntStateOutput::button(Button::A)], 2500..2750),
            StateDescription::linear_state(SoftResetProcess::Title3, vec![HuntStateOutput::button(Button::A)], 3500..3750),
            StateDescription::linear_state(SoftResetProcess::Title4, vec![HuntStateOutput::button(Button::A)], 3000..3250),
            // For Starter
            StateDescription::linear_state(SoftResetProcess::GetGift, vec![HuntStateOutput::button(Button::A)], 3500..4500),
        ];

        builder.add_states(states);
        builder
    }

    pub fn frlg_random(mut builder: HuntFSMBuilder) -> HuntFSMBuilder {
        let states = vec![
            StateDescription::choose_toggle_state(
                TryGetEncounter::Init,
                TryGetEncounter::Up,
                TryGetEncounter::Left,
            ),
            StateDescription::simple_process_state(
                TryGetEncounter::Up,
                TryGetEncounter::Entering,
                TryGetEncounter::Down,
                Processing::FRLG_START_ENCOUNTER,
                HuntStateOutput::new(Button::Up, Delay::Tenth),
                Self::MOVE_DELAY..Self::MOVE_DELAY,
            ),
            StateDescription::simple_process_state(
                TryGetEncounter::Down,
                TryGetEncounter::Entering,
                TryGetEncounter::Up,
                Processing::FRLG_START_ENCOUNTER,
                HuntStateOutput::new(Button::Down, Delay::Tenth),
                Self::MOVE_DELAY..Self::MOVE_DELAY,
            ),
            StateDescription::simple_process_state(
                TryGetEncounter::Left,
                TryGetEncounter::Entering,
                TryGetEncounter::Right,
                Processing::FRLG_START_ENCOUNTER,
                HuntStateOutput::new(Button::Left, Delay::Tenth),
                Self::MOVE_DELAY..Self::MOVE_DELAY,
            ),
            StateDescription::simple_process_state(
                TryGetEncounter::Right,
                TryGetEncounter::Entering,
                TryGetEncounter::Left,
                Processing::FRLG_START_ENCOUNTER,
                HuntStateOutput::new(Button::Right, Delay::Tenth),
                Self::MOVE_DELAY..Self::MOVE_DELAY,
            ),
            StateDescription::linear_state_no_delay(TryGetEncounter::Entering, vec![]),
        ];

        builder.add_states(states);
        builder
    }
}
