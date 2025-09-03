use chrono::{DateTime, Utc};
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
    UltraSunUltraMoon,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Method {
    RandomEncounter,
    SoftResetEncounter,
    SoftResetGift,
    SafariZone,
}

// State of application, shared between main thread and API
#[derive(Clone, Serialize)]
pub struct AppState {
    pub(crate) state: HuntState,
    pub(crate) arg: Option<TransitionArg>,
    pub(crate) encounters: u64,
    pub(crate) phases: Vec<Phase>,
    pub(crate) last_phase: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Phase {
    pub species: u32,
    pub encounters: u64,
    pub caught: bool,
    pub date: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CaptureControlPaths {
    video: String,
    control: String,
}

impl CaptureControlPaths {
    pub fn new(video: String, control: String) -> Self {
        Self { video, control }
    }

    pub fn video(&self) -> &str {
        &self.video
    }

    pub fn control(&self) -> &str {
        &self.control
    }
}

#[derive(Debug)]
pub enum Config {
    // RaspberryPi - DS Lite - V4L2 Capture, Serial Control (Pico)
    Shaoooh(CaptureControlPaths),
    // Any - 3DS - NTR Stream, InputRedirection
    Bishaan(core::net::Ipv4Addr),
    // Dummy config for testing
    Ditto, // TODO
           // Any? - DS Lite - V4L2 Capture, Serial Control (Wireless)
           //Gyaaas(CaptureControlPaths),
           // Any? - GC/Wii - V4L2 Capture, Serial Control (?)
           //ZutZutt(CaptureControlPaths),
}

impl Config {
    pub fn name(&self) -> String {
        match self {
            Self::Shaoooh(..) => "Shaoooh ショオーッ".to_string(),
            Self::Bishaan(..) => "Bishaan ビシアーン".to_string(),
            Self::Ditto => "Ditto メタモン".to_string(),
        }
    }

    pub fn info(&self) -> String {
        match self {
            Self::Shaoooh(cfg) => {
                format!(
                    "Shaoooh ショオーッ : Video({}) Control({})",
                    cfg.video(),
                    cfg.control()
                )
            }
            Self::Bishaan(ip) => {
                format!("Bishaan ビシアーン : IP({})", ip)
            }
            Self::Ditto => "Ditto メタモン : Metamon".to_string(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Shaoooh(_) => "DS-Lite with video mod".to_string(),
            Self::Bishaan(_) => "New 2DS XL".to_string(),
            Self::Ditto => "Test configuration".to_string(),
        }
    }

    pub fn emoji(&self) -> String {
        match self {
            Self::Shaoooh(_) => "🐦‍🔥".to_string(),
            Self::Bishaan(_) => "👾".to_string(),
            Self::Ditto => "🍙".to_string(),
        }
    }
}
