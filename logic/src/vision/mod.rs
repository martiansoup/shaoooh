use std::collections::HashMap;

use opencv::{
    core::{NORM_MINMAX, Point, Rect, Size, Vector},
    highgui,
    imgcodecs::{IMREAD_COLOR, IMREAD_UNCHANGED},
    imgproc::{
        COLOR_BGR2HSV, HISTCMP_CORREL, LINE_8, THRESH_BINARY, THRESH_BINARY_INV, TM_CCORR_NORMED,
    },
    prelude::*,
    videoio::VideoCapture,
};

use crate::app::states::Game;

#[derive(Debug)]
pub enum Processing {
    // List of sprites to check, and should it be flipped
    Sprite(Game, Vec<u32>, bool),
    FRLGShinyStar,
    FRLGStartEncounter,
    FRLGInEncounter,
    FRLGEncounterReady,
    DPStartEncounter,
    DPInEncounter,
    DPEncounterReady,
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
    game: Game,
    flipped: bool,
    // Reference, Shiny, Mask
    reference: HashMap<u32, (Mat, Mat, Mat)>,
    img_index: u32,
}

impl Vision {
    const WIDTH: i32 = 320;
    const HEIGHT: i32 = 240;
    const BORDER_KEEP: i32 = 0;
    const BORDER_LR: i32 = 16;
    const BORDER_TB: i32 = 20;
    const X0: i32 = Vision::BORDER_LR - Vision::BORDER_KEEP;
    const Y0: i32 = Vision::BORDER_TB - Vision::BORDER_KEEP;
    const W1: i32 = (Vision::WIDTH - (Vision::BORDER_LR * 2)) + (Vision::BORDER_KEEP * 2);
    const H1: i32 = (Vision::HEIGHT - (Vision::BORDER_TB * 2)) + (Vision::BORDER_KEEP * 2);
    const DS_W: i32 = 256;
    const DS_H: i32 = 192;
    const MAX_IMAGES: u32 = 16;

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
            reference: HashMap::new(),
            game: Game::None,
            flipped: false,
            img_index: 0,
        }
    }

    fn get_or_create_references(
        &mut self,
        game: &Game,
        flipped: &bool,
        species: u32,
    ) -> &(Mat, Mat, Mat) {
        if *game != self.game {
            self.reference.clear();
            self.game = game.clone();
        }
        if *flipped != self.flipped {
            self.reference.clear();
            self.flipped = flipped.clone();
        }
        if !self.reference.contains_key(&species) {
            let dir = match game {
                Game::FireRedLeafGreen => "frlg",
                Game::DiamondPearl => "dp",
                _ => "?", // TODO other games
            };
            // TODO check files exist as doesn't error from opencv
            let path = format!("../reference/images/{}/{:03}.png", dir, species);
            let shiny_path = format!("../reference/images/{}/{:03}_shiny.png", dir, species);

            let ref_img_raw_in =
                opencv::imgcodecs::imread(&path, IMREAD_UNCHANGED).expect("Couldn't read image");
            let ref_img_in =
                opencv::imgcodecs::imread(&path, IMREAD_COLOR).expect("Couldn't read image");
            let shi_img_in =
                opencv::imgcodecs::imread(&shiny_path, IMREAD_COLOR).expect("Couldn't read image");

            let mut ref_img_raw = Mat::default();
            let mut ref_img = Mat::default();
            let mut shi_img = Mat::default();

            if *flipped {
                opencv::core::flip(&ref_img_raw_in, &mut ref_img_raw, 1);
                opencv::core::flip(&ref_img_in, &mut ref_img, 1);
                opencv::core::flip(&shi_img_in, &mut shi_img, 1);
            } else {
                ref_img_raw = ref_img_raw_in;
                ref_img = ref_img_in;
                shi_img = shi_img_in;
            }

            let mut channels: Vector<Mat> = Default::default();
            opencv::core::split(&ref_img_raw, &mut channels);
            let alpha = channels.get(3).unwrap();

            let mut mask = Mat::default();

            opencv::imgproc::threshold(&alpha, &mut mask, 0.0, 255.0, THRESH_BINARY)
                .expect("Failed to create mask");

            self.reference.insert(species, (ref_img, shi_img, mask));
        }
        self.reference.get(&species).expect("Must be present")
    }

    fn match_sprite(
        &mut self,
        game: &Game,
        species: &Vec<u32>,
        flipped: &bool,
        frame: &Mat,
    ) -> ProcessingResult {
        let mut found_species = 0;
        let mut max = 0.0;
        let mut is_shiny_conv = false;
        let mut found_location = Point::default();
        let mut tpl_w = 0;
        let mut tpl_h = 0;

        for s in species {
            let (reference, shiny, mask) = self.get_or_create_references(game, flipped, *s);

            let mask_copy = mask.clone();
            let mut result = Mat::default();
            opencv::imgproc::match_template(
                frame,
                reference,
                &mut result,
                TM_CCORR_NORMED,
                &mask_copy,
            )
            .expect("Failed to convolve");
            let mut result_shiny = Mat::default();
            opencv::imgproc::match_template(
                frame,
                shiny,
                &mut result_shiny,
                TM_CCORR_NORMED,
                &mask_copy,
            )
            .expect("Failed to convolve");

            let mut max_val = 0.0;
            let mut max_val_shiny = 0.0;
            let mut max_loc = Point::default();
            let mut max_loc_shiny = Point::default();

            opencv::core::min_max_loc(
                &result,
                None,
                Some(&mut max_val),
                None,
                Some(&mut max_loc),
                &opencv::core::no_array(),
            )
            .expect("min max failed");
            opencv::core::min_max_loc(
                &result_shiny,
                None,
                Some(&mut max_val_shiny),
                None,
                Some(&mut max_loc_shiny),
                &opencv::core::no_array(),
            )
            .expect("min max failed");

            log::info!("species = {}, val = {}", s, max_val);

            if max_val > max {
                max = max_val;
                found_species = *s;
                is_shiny_conv = false;
                found_location = max_loc;
                tpl_w = reference.cols();
                tpl_h = reference.rows();
            }
            if max_val_shiny > max {
                max = max_val_shiny;
                found_species = *s;
                is_shiny_conv = true;
                found_location = max_loc_shiny;
                tpl_w = shiny.cols();
                tpl_h = shiny.rows();
            }
        }

        let mut for_rect = frame.clone();
        let rect = Rect {
            x: found_location.x,
            y: found_location.y,
            width: tpl_w,
            height: tpl_h,
        };

        opencv::imgproc::rectangle(&mut for_rect, rect, 0.0.into(), 1, LINE_8, 0);
        let filename = format!("hunts/{:03}.png", self.img_index);
        opencv::imgcodecs::imwrite(&filename, &for_rect, &Vector::new());
        self.img_index += 1;
        if self.img_index >= Self::MAX_IMAGES {
            self.img_index = 0; // Reset index after reaching max
        }
        // Display current find TODO should this be included?
        highgui::imshow("found", &for_rect).expect("Failed to show rectangle");

        let is_shiny = is_shiny_conv;
        let res = ProcessingResult {
            process: Processing::Sprite(game.clone(), species.clone(), *flipped),
            met: found_species != 0,
            species: found_species,
            shiny: is_shiny,
        };
        log::info!("Process results {:?}", res);
        res
    }

    fn frlg_shiny_star(&mut self, frame: &Mat) -> ProcessingResult {
        // X, Y, Width, Height
        let star = frame
            .roi(opencv::core::Rect::new(106, 52, 16, 16))
            .expect("Failed to crop")
            .clone_pointee();
        let mut star_grey = Mat::default();
        opencv::imgproc::cvt_color(&star, &mut star_grey, opencv::imgproc::COLOR_BGR2GRAY, 0);
        let mut star_thr_w = Mat::default();
        opencv::imgproc::threshold(&star_grey, &mut star_thr_w, 200.0, 255.0, THRESH_BINARY);

        let shiny = opencv::core::count_non_zero(&star_thr_w).unwrap() < 190;

        ProcessingResult {
            process: Processing::FRLGShinyStar,
            met: shiny,
            species: 0,
            shiny,
        }
    }

    fn frlg_encounter_ready(&mut self, frame: &Mat) -> ProcessingResult {
        unimplemented!();
    }

    fn frlg_in_encounter(&mut self, frame: &Mat) -> ProcessingResult {
        unimplemented!();
    }

    fn frlg_start_encounter(&mut self, frame: &Mat) -> ProcessingResult {
        unimplemented!();
    }

    fn dp_encounter_ready(&mut self, frame: &Mat) -> ProcessingResult {
        // X, Y, Width, Height
        let hp_bar = frame
            .roi(opencv::core::Rect::new(150, 100, 106, 35))
            .expect("Failed to crop")
            .clone_pointee();
        let mut hp_bar_grey = Mat::default();
        opencv::imgproc::cvt_color(
            &hp_bar,
            &mut hp_bar_grey,
            opencv::imgproc::COLOR_BGR2GRAY,
            0,
        );
        let mut hp_bar_thr_w = Mat::default();
        opencv::imgproc::threshold(&hp_bar_grey, &mut hp_bar_thr_w, 210.0, 255.0, THRESH_BINARY);

        let ready = opencv::core::count_non_zero(&hp_bar_thr_w).unwrap() > 1500;

        ProcessingResult {
            process: Processing::DPEncounterReady,
            met: ready,
            species: 0,
            shiny: false,
        }
    }

    fn dp_in_encounter(&mut self, frame: &Mat) -> ProcessingResult {
        let bottom_bar = frame
            .roi(opencv::core::Rect::new(0, 145, 256, 47))
            .expect("Failed to crop")
            .clone_pointee();
        let mut bottom_bar_grey = Mat::default();
        opencv::imgproc::cvt_color(
            &bottom_bar,
            &mut bottom_bar_grey,
            opencv::imgproc::COLOR_BGR2GRAY,
            0,
        );
        let mut bottom_bar_thr_w = Mat::default();
        opencv::imgproc::threshold(
            &bottom_bar_grey,
            &mut bottom_bar_thr_w,
            210.0,
            255.0,
            THRESH_BINARY,
        );

        let in_enc = opencv::core::count_non_zero(&bottom_bar_thr_w).unwrap() > 6500;

        ProcessingResult {
            process: Processing::DPInEncounter,
            met: in_enc,
            species: 0,
            shiny: false,
        }
    }

    fn dp_start_encounter(&mut self, frame: &Mat) -> ProcessingResult {
        let bottom_bar = frame
            .roi(opencv::core::Rect::new(0, 145, 256, 47))
            .expect("Failed to crop")
            .clone_pointee();
        let mut bottom_bar_grey = Mat::default();
        opencv::imgproc::cvt_color(
            &bottom_bar,
            &mut bottom_bar_grey,
            opencv::imgproc::COLOR_BGR2GRAY,
            0,
        );
        let mut bottom_bar_thr_b = Mat::default();
        opencv::imgproc::threshold(
            &bottom_bar_grey,
            &mut bottom_bar_thr_b,
            40.0,
            255.0,
            THRESH_BINARY_INV,
        );

        let start = opencv::core::count_non_zero(&bottom_bar_thr_b).unwrap() > 10000;

        ProcessingResult {
            process: Processing::DPStartEncounter,
            met: start,
            species: 0,
            shiny: false,
        }
    }

    fn process(&mut self, process: &Processing, frame: &Mat) -> ProcessingResult {
        match process {
            Processing::Sprite(game, species_list, flipped) => {
                self.match_sprite(game, species_list, flipped, frame)
            }
            Processing::FRLGShinyStar => self.frlg_shiny_star(frame),
            Processing::FRLGEncounterReady => self.frlg_encounter_ready(frame),
            Processing::FRLGInEncounter => self.frlg_in_encounter(frame),
            Processing::FRLGStartEncounter => self.frlg_start_encounter(frame),
            Processing::DPEncounterReady => self.dp_encounter_ready(frame),
            Processing::DPInEncounter => self.dp_in_encounter(frame),
            Processing::DPStartEncounter => self.dp_start_encounter(frame),
        }
    }

    pub fn process_next_frame(&mut self, processing: Vec<Processing>) -> Vec<ProcessingResult> {
        let mut input_frame = Mat::default();
        self.cam
            .read(&mut input_frame)
            .expect("Failed to read frame");
        let unsized_frame = input_frame
            .roi(opencv::core::Rect::new(
                Self::X0,
                Self::Y0,
                Self::W1,
                Self::H1,
            ))
            .expect("Failed to crop")
            .clone_pointee();
        let mut frame = Mat::default();
        opencv::imgproc::resize(
            &unsized_frame,
            &mut frame,
            Size::new(Self::DS_W, Self::DS_H),
            0.0,
            0.0,
            0,
        );

        // Save to encoded frame
        opencv::imgcodecs::imencode(".png", &frame, &mut self.encoded, &Vector::new())
            .expect("Failed to encode frame");

        // TODO don't show gui ?
        highgui::imshow("capture", &frame).expect("Failed to show capture");
        highgui::wait_key(1).expect("Event loop failed");

        processing.iter().map(|p| self.process(p, &frame)).collect()
    }

    pub fn read_frame(&self) -> &[u8] {
        self.encoded.as_slice()
    }
}
