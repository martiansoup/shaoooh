use opencv::{core::{Size, Vector}, highgui, prelude::*, videoio::VideoCapture};

use crate::app::states::Game;

#[derive(Debug)]
pub enum Processing {
    // List of sprites to check, and should it be flipped
    Sprite(Game, Vec<u32>, bool),
    DPStartEncounter,
    DPInEncounter,
    DPEncounterReady
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub(crate) process: Processing,
    pub(crate) met: bool,
    pub(crate) species: u32,
    pub(crate) shiny: bool,
}

pub struct Vision {
    cam: VideoCapture,
    encoded: Vector<u8>,
}

impl Vision {
    const WIDTH: i32 = 320;
    const HEIGHT: i32 = 240;
    const BORDER_KEEP: i32 = 0;
    const BORDER_LR: i32 = 16;
    const BORDER_TB: i32 = 20;
    const X0: i32 = Vision::BORDER_LR - Vision::BORDER_KEEP;
    const Y0: i32 = Vision::BORDER_TB - Vision::BORDER_KEEP;
    const W1: i32 = (Vision::WIDTH  - (Vision::BORDER_LR * 2) ) + (Vision::BORDER_KEEP * 2);
    const H1: i32 = (Vision::HEIGHT - (Vision::BORDER_TB * 2) ) + (Vision::BORDER_KEEP * 2);
    const DS_W : i32 = 256;
    const DS_H : i32 = 192;

    pub fn new() -> Self {
        log::info!("Starting video capture");
        let mut cam = VideoCapture::from_file("/dev/video0", 0).expect("Couldn't open video");

        cam.set(opencv::videoio::CAP_PROP_BRIGHTNESS, 50.0)
            .expect("Failed to set property");
        cam.set(opencv::videoio::CAP_PROP_FRAME_WIDTH, Vision::WIDTH.into())
            .expect("Failed to set property");
        cam.set(
            opencv::videoio::CAP_PROP_FRAME_HEIGHT,
            Vision::HEIGHT.into(),
        )
        .expect("Failed to set property");
        Self {
            cam,
            encoded: Vector::default(),
        }
    }

    fn match_sprite(&mut self, game: &Game, species: &Vec<u32>, flipped: &bool, frame: &Mat) -> ProcessingResult {
        // TODO actually process
        ProcessingResult { process: Processing::Sprite(game.clone(), species.clone(), *flipped), met: true, species: 1, shiny: false }
    }

    fn dp_encounter_ready(&mut self, frame: &Mat) -> ProcessingResult {
        // TODO actually process
        ProcessingResult { process: Processing::DPEncounterReady, met: false, species: 0, shiny: false }
    }

    fn dp_in_encounter(&mut self, frame: &Mat) -> ProcessingResult {
        // TODO actually process
        ProcessingResult { process: Processing::DPInEncounter, met: false, species: 0, shiny: false }
    }

    fn dp_start_encounter(&mut self, frame: &Mat) -> ProcessingResult {
        // TODO actually process
        ProcessingResult { process: Processing::DPStartEncounter, met: false, species: 0, shiny: false }
    }

    fn process(&mut self, process: &Processing, frame: &Mat) -> ProcessingResult {
        log::info!("Processing {:?}", process);
        let res = match process {
            Processing::Sprite(game, species_list, flipped) => self.match_sprite(game, species_list, flipped, frame),
            Processing::DPEncounterReady => self.dp_encounter_ready(frame),
            Processing::DPInEncounter => self.dp_in_encounter(frame),
            Processing::DPStartEncounter => self.dp_start_encounter(frame)
        };
        log::info!("Process results {:?}", res);
        res
    }

    pub fn process_next_frame(&mut self, processing: Vec<Processing>) -> Vec<ProcessingResult> {
        let mut input_frame = Mat::default();
        self.cam.read(&mut input_frame).expect("Failed to read frame");
        let unsized_frame = input_frame.roi(opencv::core::Rect::new(Self::X0, Self::Y0, Self::W1, Self::H1)).expect("Failed to crop").clone_pointee();
        let mut frame = Mat::default();
        opencv::imgproc::resize(&unsized_frame, &mut frame, Size::new(Self::DS_W, Self::DS_H), 0.0, 0.0, 0);

        // Save to encoded frame
        opencv::imgcodecs::imencode(".png", &frame, &mut self.encoded, &Vector::new())
            .expect("Failed to encode frame");

        // TODO Do per-frame processing
        // TODO don't show gui ?
        highgui::imshow("capture", &frame).expect("Failed to show capture");
        highgui::wait_key(1).expect("Event loop failed");

        processing.iter().map(|p| self.process(p, &frame) ).collect()
    }

    pub fn read_frame(&self) -> &[u8] {
        self.encoded.as_slice()
    }
}
