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

pub struct EncounterTypeResolver {}

impl EncounterTypeResolver {
    const MOVE_DELAY : u64 = 75;

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
