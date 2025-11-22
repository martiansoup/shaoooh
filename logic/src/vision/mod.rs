use crate::app::states::Game;

mod bishaan_vision;
pub mod compat;
mod ds_vision;
mod nop_vision;
mod ntr;
mod utils;

pub use bishaan_vision::{BishaanVision, BishaanVisionSocket};
pub use ds_vision::Vision;
pub use nop_vision::NopVision;
pub use ntr::NTRPacket;

#[derive(PartialEq, Clone, Debug)]
pub struct RegionDetectSettings {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub col_thresh: f64,
    pub num_thresh: i32,
    pub invert: bool,
}

#[derive(PartialEq, Clone, Debug)]
pub enum ColourChannel {
    Blue,
    Green,
    Red,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ColourChannelDetectSettings {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub col_thresh: f64,
    pub num_thresh: i32,
    pub invert: bool,
    pub colour: ColourChannel,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ColourChannelDetect3DSSettings {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub col_thresh: f64,
    pub num_thresh: i32,
    pub invert: bool,
    pub colour: ColourChannel,
    pub top: bool,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ChannelDetectSettings {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub h_lo: f64,
    pub s_lo: f64,
    pub v_lo: f64,
    pub h_hi: f64,
    pub s_hi: f64,
    pub v_hi: f64,
    pub num_thresh: i32,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Processing {
    // List of sprites to check, and should it be flipped
    Sprite(Game, Vec<u32>, bool),
    Sprite3DS(Game, Vec<u32>),
    RegionDetect(RegionDetectSettings),
    ColourChannelDetect(ColourChannelDetectSettings),
    ColourChannelDetect3DS(ColourChannelDetect3DSSettings),
    ChannelDetect(ChannelDetectSettings),
    USUMShinyStar(u32),
    USUMBottomScreen(f64),
    USUMBottomScreenInv(f64),
}

impl Processing {
    pub const BW2_BLACK_SCREEN: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 0,
        y: 0,
        w: 256,
        h: 192,
        col_thresh: 15.0,
        num_thresh: 45000,
        invert: true,
    });
    pub const BW2_WHITE_SCREEN: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 0,
        y: 0,
        w: 256,
        h: 192,
        col_thresh: 210.0,
        num_thresh: 45000,
        invert: false,
    });
    pub const BW2_BAR_PRESENT: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 0,
        y: 150,
        w: 256,
        h: 40,
        col_thresh: 75.0,
        num_thresh: 7000,
        invert: true,
    });
    pub const BW2_BAR_NEGATE_CONFIRM: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 0,
        y: 0,
        w: 256,
        h: 40,
        col_thresh: 50.0,
        num_thresh: 5000,
        invert: true,
    });
    pub const DP_START_ENCOUNTER_WHITE: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 0,
        y: 0,
        w: 256,
        h: 192,
        col_thresh: 210.0,
        num_thresh: 20000,
        invert: false,
    });
    pub const DP_START_ENCOUNTER: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 0,
        y: 145,
        w: 256,
        h: 47,
        col_thresh: 40.0,
        num_thresh: 10000,
        invert: true,
    });
    pub const HGSS_BLACK_SCREEN: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 0,
        y: 0,
        w: 256,
        h: 192,
        col_thresh: 40.0,
        num_thresh: 49000,
        invert: true,
    });
    pub const DP_IN_ENCOUNTER: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 0,
        y: 145,
        w: 256,
        h: 47,
        col_thresh: 210.0,
        num_thresh: 6500,
        invert: false,
    });
    pub const DP_ENCOUNTER_READY: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 150,
        y: 100,
        w: 106,
        h: 35,
        col_thresh: 210.0,
        num_thresh: 1500,
        invert: false,
    });
    pub const HGSS_ENCOUNTER_READY: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 150,
        y: 100,
        w: 106,
        h: 35,
        col_thresh: 150.0,
        num_thresh: 1500,
        invert: false,
    });
    pub const DP_SAFARI_ENCOUNTER_READY: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 150,
        y: 100,
        w: 106,
        h: 35,
        col_thresh: 195.0,
        num_thresh: 1400,
        invert: false,
    });
    pub const FRLG_SHINY_STAR: Self =
        Processing::ColourChannelDetect(ColourChannelDetectSettings {
            x: 106,
            y: 52,
            w: 16,
            h: 16,
            col_thresh: 150.0,
            num_thresh: 5,
            invert: true,
            colour: ColourChannel::Blue,
        });
    pub const USUM_SHINY_STAR: Self =
        Processing::ColourChannelDetect3DS(ColourChannelDetect3DSSettings {
            x: 54,
            y: 181,
            w: 17,
            h: 17,
            col_thresh: 150.0,
            num_thresh: 5,
            invert: true,
            colour: ColourChannel::Green,
            top: false,
        });
    pub const FRLG_SHINY_STAR_OLD: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 106,
        y: 52,
        w: 16,
        h: 16,
        col_thresh: 200.0,
        num_thresh: 190,
        invert: false,
    });
    pub const FRLG_START_ENCOUNTER: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 20,
        y: 140,
        w: 215,
        h: 30,
        col_thresh: 40.0,
        num_thresh: 6000,
        invert: true,
    });
    pub const FRLG_IN_ENCOUNTER: Self = Processing::RegionDetect(RegionDetectSettings {
        x: 20,
        y: 140,
        w: 215,
        h: 30,
        col_thresh: 55.0,
        num_thresh: 5000,
        invert: false,
    });
    // TODO FRLG/RS differences
    pub const FRLG_ENCOUNTER_READY: Self = Processing::ChannelDetect(ChannelDetectSettings {
        x: 20,
        y: 140,
        w: 215,
        h: 30,
        h_lo: 0.0,
        s_lo: 60.0,
        v_lo: 100.0,
        h_hi: 40.0,
        s_hi: 255.0,
        v_hi: 255.0,
        num_thresh: 10,
    });
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub process: Processing,
    pub met: bool,
    pub species: u32,
    pub shiny: bool,
}

struct WinInfo {
    name: &'static str,
    x: i32,
    y: i32,
    scale: i32,
}

pub trait BotVision {
    fn process_next_frame(&mut self, processing: &[Processing]) -> Option<Vec<ProcessingResult>>;
    fn read_frame(&self) -> &[u8];
    fn read_frame2(&self) -> &[u8];
}
