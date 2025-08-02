use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Transition {
    StartHunt,
    PauseHunt,
    FoundNonTarget,
    FoundTarget,
    Fail,
    Caught,
    FalseDetect,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
pub enum HuntState {
    Idle,
    Hunt,
    FoundNonTarget,
    FoundTarget,
}

#[derive(Clone, Serialize, Debug)]
pub struct StateTransition {
    pub(crate) transition: Transition,
    pub(crate) next_state: HuntState,
    pub(crate) needs_arg: bool,
    pub(crate) automatic: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RequestTransition {
    pub(crate) transition: Transition,
    pub(crate) arg: Option<TransitionArg>,
}

impl HuntState {
    pub fn possible_transitions(&self) -> Vec<StateTransition> {
        match self {
            Self::Idle => vec![StateTransition {
                transition: Transition::StartHunt,
                next_state: Self::Hunt,
                needs_arg: true,
                automatic: false,
            }],
            Self::Hunt => vec![
                StateTransition {
                    transition: Transition::PauseHunt,
                    next_state: Self::Idle,
                    needs_arg: false,
                    automatic: false,
                },
                StateTransition {
                    transition: Transition::FoundNonTarget,
                    next_state: Self::FoundNonTarget,
                    needs_arg: true,
                    automatic: true,
                },
                StateTransition {
                    transition: Transition::FoundTarget,
                    next_state: Self::FoundTarget,
                    needs_arg: false,
                    automatic: true,
                },
            ],
            Self::FoundNonTarget => vec![
                StateTransition {
                    transition: Transition::Caught,
                    next_state: Self::Hunt,
                    needs_arg: false,
                    automatic: false,
                },
                StateTransition {
                    transition: Transition::Fail,
                    next_state: Self::Hunt,
                    needs_arg: false,
                    automatic: false,
                },
                StateTransition {
                    transition: Transition::FalseDetect,
                    next_state: Self::Hunt,
                    needs_arg: false,
                    automatic: false,
                },
            ],
            Self::FoundTarget => vec![
                StateTransition {
                    transition: Transition::Caught,
                    next_state: Self::Idle,
                    needs_arg: false,
                    automatic: false,
                },
                StateTransition {
                    transition: Transition::Fail,
                    next_state: Self::Hunt,
                    needs_arg: false,
                    automatic: false,
                },
                StateTransition {
                    transition: Transition::FalseDetect,
                    next_state: Self::Hunt,
                    needs_arg: false,
                    automatic: false,
                },
            ],
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct TransitionArg {
    pub(crate) name: String,
    pub(crate) species: u32,
    pub(crate) game: Game,
    pub(crate) method: Method,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Game {
    None,
    RubySapphire,
    Emerald,
    FireRedLeafGreen,
    DiamondPearl,
    Platinum,
    HeartGoldSoulSilver,
    BlackWhite,
    Black2White2,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Method {
    RandomEncounter,
    SoftResetEncounter,
    SoftResetGift,
    SafariZone,
}

// State of application, shared between main thread and API
// TODO add dates
#[derive(Clone, Serialize)]
pub(crate) struct AppState {
    pub(crate) state: HuntState,
    pub(crate) arg: Option<TransitionArg>,
    pub(crate) encounters: u64,
    pub(crate) phases: Vec<Phase>,
    pub(crate) last_phase: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Phase {
    pub(crate) species: u32,
    pub(crate) encounters: u64,
    pub(crate) caught: bool,
}
