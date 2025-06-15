use std::collections::HashMap;

use opencv::{
    core::{NORM_MINMAX, Point, Rect, Size, Vector},
    highgui,
    imgcodecs::{IMREAD_COLOR, IMREAD_UNCHANGED},
    imgproc::{
        COLOR_BGR2HSV, HISTCMP_CORREL, LINE_8, THRESH_BINARY, THRESH_BINARY_INV, TM_CCORR_NORMED,
    },
    prelude::*,
    videoio::{CAP_V4L2, VideoCapture},
};

use crate::app::{Shaoooh, states::Game};

#[derive(PartialEq, Clone, Debug)]
pub struct RegionDetectSettings {
    pub x : i32,
    pub y : i32,
    pub w : i32,
    pub h : i32,
    pub col_thresh : f64,
    pub num_thresh : i32,
    pub invert : bool
}

#[derive(PartialEq, Clone, Debug)]
pub struct ChannelDetectSettings {
    pub x : i32,
    pub y : i32,
    pub w : i32,
    pub h : i32,
    pub h_lo : f64,
    pub s_lo : f64,
    pub v_lo : f64,
    pub h_hi : f64,
    pub s_hi : f64,
    pub v_hi : f64,
    pub num_thresh : i32
}

#[derive(Debug, PartialEq)]
pub enum Processing {
    // List of sprites to check, and should it be flipped
    Sprite(Game, Vec<u32>, bool),
    RegionDetect(RegionDetectSettings),
    ChannelDetect(ChannelDetectSettings),
}

impl Processing {
    pub const DP_START_ENCOUNTER : Self = Processing::RegionDetect(RegionDetectSettings { x: 0, y: 145, w: 256, h: 47, col_thresh: 40.0, num_thresh: 10000, invert: true });
    pub const DP_IN_ENCOUNTER : Self = Processing::RegionDetect(RegionDetectSettings { x: 0, y: 145, w: 256, h: 47, col_thresh: 210.0, num_thresh: 6500, invert: false });
    pub const DP_ENCOUNTER_READY : Self = Processing::RegionDetect(RegionDetectSettings { x: 150, y: 100, w: 106, h: 35, col_thresh: 210.0, num_thresh: 1500, invert: false });
    pub const FRLG_SHINY_STAR : Self = Processing::RegionDetect(RegionDetectSettings { x: 106, y: 52, w: 16, h: 16, col_thresh: 200.0, num_thresh: 190, invert: false });
    pub const FRLG_START_ENCOUNTER : Self = Processing::RegionDetect(RegionDetectSettings { x: 20, y: 140, w: 215, h: 30, col_thresh: 40.0, num_thresh:  6000, invert: true });
    pub const FRLG_IN_ENCOUNTER : Self = Processing::RegionDetect(RegionDetectSettings { x: 20, y: 140, w: 215, h: 30, col_thresh: 55.0, num_thresh: 5000, invert: false });
    pub const FRLG_ENCOUNTER_READY : Self = Processing::ChannelDetect(ChannelDetectSettings { x: 20, y: 140, w: 215, h: 30, h_lo: 0.0, s_lo: 100.0, v_lo: 150.0, h_hi: 20.0, s_hi: 255.0, v_hi: 255.0, num_thresh: 10 });
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
        let mut cam =
            VideoCapture::from_file(Shaoooh::VIDEO_DEV, CAP_V4L2).expect("Couldn't open video");
        log::debug!("Video capture opened");

        cam.set(opencv::videoio::CAP_PROP_READ_TIMEOUT_MSEC, 2000.0)
            .expect("Failed to set property");

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

    fn region_detect(&mut self, settings: &RegionDetectSettings, frame: &Mat) -> ProcessingResult {
        let region = frame.roi(opencv::core::Rect::new(settings.x, settings.y, settings.w, settings.h))
        .expect("Failed to crop to region of interest")
        .clone_pointee();
        let mut greyscale = Mat::default();
        opencv::imgproc::cvt_color(&region, &mut greyscale, opencv::imgproc::COLOR_BGR2GRAY, 0);
        let mut thresholded = Mat::default();
        let typ = if settings.invert { THRESH_BINARY_INV } else { THRESH_BINARY };
        opencv::imgproc::threshold(&greyscale, &mut thresholded, settings.col_thresh, 255.0, typ);

        let met = opencv::core::count_non_zero(&thresholded).unwrap() > settings.num_thresh;

        ProcessingResult {
            process: Processing::RegionDetect(settings.clone()),
            met,
            species: 0,
            shiny: false
        }
    }

    fn channel_detect(&mut self, settings: &ChannelDetectSettings, frame: &Mat) -> ProcessingResult {
        let region = frame.roi(opencv::core::Rect::new(settings.x, settings.y, settings.w, settings.h))
        .expect("Failed to crop to region of interest")
        .clone_pointee();
        let mut hsv = Mat::default();
        opencv::imgproc::cvt_color(&region, &mut hsv, opencv::imgproc::COLOR_BGR2HSV, 0);
        let mut thresholded = Mat::default();
        let lower = Vector::from_slice(&vec![settings.h_lo, settings.s_lo, settings.v_lo]);
        let upper = Vector::from_slice(&vec![settings.h_hi, settings.s_hi, settings.v_hi]);
        opencv::core::in_range(&hsv, &lower, &upper, &mut thresholded);

        let count = opencv::core::count_non_zero(&thresholded).unwrap();
        let met = count > settings.num_thresh;

        ProcessingResult {
            process: Processing::ChannelDetect(settings.clone()),
            met,
            species: 0,
            shiny: false
        }
    }

    fn process(&mut self, process: &Processing, frame: &Mat) -> ProcessingResult {
        match process {
            Processing::Sprite(game, species_list, flipped) => {
                self.match_sprite(game, species_list, flipped, frame)
            }
            Processing::ChannelDetect(settings) => self.channel_detect(settings, frame),
            Processing::RegionDetect(settings) => self.region_detect(settings, frame)
        }
    }

    pub fn process_next_frame(
        &mut self,
        processing: Vec<Processing>,
    ) -> Result<Vec<ProcessingResult>, ()> {
        let mut input_frame = Mat::default();
        self.cam
            .read(&mut input_frame)
            .expect("Failed to read frame");
        if input_frame.empty() {
            return Err(());
        }
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

        Ok(processing.iter().map(|p| self.process(p, &frame)).collect())
    }

    pub fn read_frame(&self) -> &[u8] {
        self.encoded.as_slice()
    }
}
