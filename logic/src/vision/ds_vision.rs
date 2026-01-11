use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use opencv::{
    core::{Point, Rect, Size, ToInputArray, Vector},
    highgui::{self, WINDOW_GUI_NORMAL, WINDOW_KEEPRATIO, WINDOW_NORMAL},
    imgcodecs::{IMREAD_COLOR, IMREAD_UNCHANGED},
    imgproc::{LINE_8, THRESH_BINARY, THRESH_BINARY_INV, TM_CCORR_NORMED},
    prelude::*,
    videoio::{CAP_V4L2, VideoCapture},
};

use crate::{
    app::states::Game,
    context::PkContext,
    vision::{
        BotVision, ChannelDetectSettings, ColourChannel, ColourChannelDetectSettings, Processing,
        ProcessingResult, RegionDetectSettings, WinInfo, compat,
    },
};

pub struct Vision {
    cam: VideoCapture,
    encoded: Vector<u8>,
    found: Vector<u8>,
    found_mat: Mat,
    found_updated: bool,
    raw_frame: Arc<Mutex<Mat>>,
    game: Game,
    flipped: bool,
    // Reference, Shiny, Mask
    reference: HashMap<u32, (Mat, Mat, Mat)>,
    img_index: u32,
    enable_debug: bool, // TODO control image/window debug separately
}

impl BotVision for Vision {
    fn process_next_frame(&mut self, processing: &[Processing]) -> Option<Vec<ProcessingResult>> {
        let mut input_frame = Mat::default();
        self.cam
            .read(&mut input_frame)
            .expect("Failed to read frame");
        if input_frame.empty() {
            return None;
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
        )
        .expect("Failed to resize image");

        if let Ok(mut f) = self.raw_frame.lock() {
            *f = frame.clone();
        }

        // Save to encoded frame
        opencv::imgcodecs::imencode(".png", &frame, &mut self.encoded, &Vector::new())
            .expect("Failed to encode frame");

        Self::show_window(Self::CAPTURE_WIN, &frame);
        Self::transform_window(Self::CAPTURE_WIN);
        highgui::wait_key(1).expect("Event loop failed");

        Some(processing.iter().map(|p| self.process(p, &frame)).collect())
    }

    fn read_frame(&self) -> &[u8] {
        self.encoded.as_slice()
    }

    fn read_frame2(&self) -> &[u8] {
        self.encoded.as_slice()
    }

    fn read_found(&mut self) -> &[u8] {
        self.found_updated = false;
        self.found.as_slice()
    }

    fn new_found(&self) -> bool {
        self.found_updated
    }
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
    const MAX_IMAGES: u32 = 256;
    const CAPTURE_WIN: WinInfo = WinInfo {
        name: "capture",
        x: 32,
        y: 32,
        scale: 2,
    };
    const FOUND_WIN: WinInfo = WinInfo {
        name: "found",
        x: 32,
        y: 464,
        scale: 1,
    };
    const FOUND_LAST_WIN: WinInfo = WinInfo {
        name: "found_last",
        x: 32 + 256,
        y: 464,
        scale: 1,
    };

    fn show_window(win: WinInfo, mat: &impl ToInputArray) {
        highgui::imshow(win.name, mat)
            .unwrap_or_else(|_| panic!("Failed to show '{}' window", win.name));
    }

    fn transform_window(win: WinInfo) {
        opencv::highgui::move_window(win.name, win.x, win.y)
            .unwrap_or_else(|_| panic!("Failed to move '{}' window", win.name));
        opencv::highgui::resize_window(win.name, Self::DS_W * win.scale, Self::DS_H * win.scale)
            .unwrap_or_else(|_| panic!("Failed to resize '{}' window", win.name));
    }

    fn create_window(win: WinInfo, en_dbg: bool) {
        let flags = if en_dbg {
            0
        } else {
            WINDOW_NORMAL | WINDOW_KEEPRATIO | WINDOW_GUI_NORMAL
        };
        opencv::highgui::named_window(win.name, flags)
            .unwrap_or_else(|_| panic!("Failed to create '{}' window", win.name));
        Self::transform_window(win);
    }

    pub fn new(path: &str, raw_frame_mutex: Arc<Mutex<Mat>>) -> Self {
        log::info!("Starting video capture");
        let mut cam = VideoCapture::from_file(path, CAP_V4L2).expect("Couldn't open video");
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

        // TODO allow debug mode without window flags
        log::info!("Opening windows");
        Self::create_window(Self::CAPTURE_WIN, false);
        Self::create_window(Self::FOUND_WIN, false);
        Self::create_window(Self::FOUND_LAST_WIN, false);
        highgui::wait_key(1).expect("Event loop failed");

        Self {
            cam,
            encoded: Vector::default(),
            found: Vector::default(),
            found_mat: Mat::default(),
            found_updated: false,
            raw_frame: raw_frame_mutex,
            reference: HashMap::new(),
            game: Game::None,
            flipped: false,
            img_index: 0,
            enable_debug: false,
        }
    }

    fn create_reference(game: &Game, flipped: &bool, species: u32) -> (Mat, Mat, Mat) {
        let path_png = PkContext::get().sprite_path(game, species, false);
        let path = if std::fs::exists(&path_png).unwrap() {
            path_png
        } else {
            panic!("Couldn't get reference image {}", path_png)
        };
        let shiny_path_png = PkContext::get().sprite_path(game, species, true);
        let shiny_path = if std::fs::exists(&shiny_path_png).unwrap() {
            shiny_path_png
        } else {
            panic!("Couldn't get reference image {}", shiny_path_png)
        };

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
            opencv::core::flip(&ref_img_raw_in, &mut ref_img_raw, 1).expect("Failed to flip image");
            opencv::core::flip(&ref_img_in, &mut ref_img, 1).expect("Failed to flip image");
            opencv::core::flip(&shi_img_in, &mut shi_img, 1).expect("Failed to flip image");
        } else {
            ref_img_raw = ref_img_raw_in;
            ref_img = ref_img_in;
            shi_img = shi_img_in;
        }

        let mut channels: Vector<Mat> = Default::default();
        opencv::core::split(&ref_img_raw, &mut channels).expect("Failed to split channels");
        let alpha = channels.get(3).unwrap();

        let mut mask = Mat::default();

        opencv::imgproc::threshold(&alpha, &mut mask, 0.0, 255.0, THRESH_BINARY)
            .expect("Failed to create mask");

        (ref_img, shi_img, mask)
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
            self.flipped = *flipped;
        }
        self.reference
            .entry(species)
            .or_insert_with(|| Self::create_reference(game, flipped, species));
        self.reference.get(&species).expect("Must be present")
    }

    // TODO add a dummy "Detect" processing that just updates found sprite
    // TODO use utils common version
    fn match_sprite(
        &mut self,
        game: &Game,
        species: &Vec<u32>,
        flipped: &bool,
        frame: &Mat,
        threshold: f64
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

            log::info!(
                "species = {}, val = {} (shiny = {})",
                s,
                max_val,
                max_val_shiny
            );

            if max_val > max {
                max = max_val;
                found_species = *s;
                is_shiny_conv = false;
                found_location = max_loc;
                tpl_w = reference.cols();
                tpl_h = reference.rows();
            }
            if max_val_shiny > max && ((max_val_shiny - max_val) > threshold) {
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

        opencv::imgproc::rectangle(&mut for_rect, rect, 0.0.into(), 1, LINE_8, 0)
            .expect("Failed to select rectangle");
        // TODO allow controlling enabling/disabling dumping images - also dump without rect for testing
        if self.enable_debug {
            let filename = format!("hunts/{:03}.png", self.img_index);
            opencv::imgcodecs::imwrite(&filename, &for_rect, &Vector::new())
                .expect("Failed to write debug image");
            self.img_index += 1;
            if self.img_index >= Self::MAX_IMAGES {
                self.img_index = 0; // Reset index after reaching max
            }
        }

        let _ = self.set_found(&for_rect, false);

        let is_shiny = is_shiny_conv;
        let process = if threshold == 0.0 {
            Processing::Sprite(game.clone(), species.clone(), *flipped)
        } else {
            Processing::SpriteT(game.clone(), species.clone(), *flipped, threshold)
        };
        let res = ProcessingResult {
            process,
            met: found_species != 0,
            species: found_species,
            shiny: is_shiny,
        };
        log::info!("Process results {:?}", res);
        res
    }

    fn region_detect(&mut self, settings: &RegionDetectSettings, frame: &Mat) -> ProcessingResult {
        let region = frame
            .roi(opencv::core::Rect::new(
                settings.x, settings.y, settings.w, settings.h,
            ))
            .expect("Failed to crop to region of interest")
            .clone_pointee();
        let mut greyscale = Mat::default();
        compat::cvt_color(&region, &mut greyscale, opencv::imgproc::COLOR_BGR2GRAY, 0)
            .expect("Failed to convert colour");
        let mut thresholded = Mat::default();
        let typ = if settings.invert {
            THRESH_BINARY_INV
        } else {
            THRESH_BINARY
        };
        opencv::imgproc::threshold(
            &greyscale,
            &mut thresholded,
            settings.col_thresh,
            255.0,
            typ,
        )
        .expect("Failed to apply threshold");

        let count = opencv::core::count_non_zero(&thresholded).unwrap();
        let met = count > settings.num_thresh;
        //if settings.invert {
        //    log::info!("got {} / {}", count, settings.num_thresh);
        //    highgui::imshow("FISH", &thresholded);
        //}
        // for i in 150..250 {
        //    let mut thresholded = Mat::default();

        //    opencv::imgproc::threshold(
        //        &greyscale,
        //        &mut thresholded,
        //        i.into(),
        //        255.0,
        //        typ,
        //    )
        //    .expect("Failed to apply threshold");

        //    let count = opencv::core::count_non_zero(&thresholded).unwrap();
        //    log::info!("at {} got {}", i, count);
        // }

        ProcessingResult {
            process: Processing::RegionDetect(settings.clone()),
            met,
            species: 0,
            shiny: false,
        }
    }

    fn colour_channel_detect(
        &mut self,
        settings: &ColourChannelDetectSettings,
        frame: &Mat,
    ) -> ProcessingResult {
        let region = frame
            .roi(opencv::core::Rect::new(
                settings.x, settings.y, settings.w, settings.h,
            ))
            .expect("Failed to crop to region of interest")
            .clone_pointee();
        let mut greyscale = Mat::default();
        let coi = match settings.colour {
            ColourChannel::Blue => 0,
            ColourChannel::Green => 1,
            ColourChannel::Red => 2,
        };
        opencv::core::extract_channel(&region, &mut greyscale, coi)
            .expect("Failed to extract colour");
        let mut thresholded = Mat::default();
        let typ = if settings.invert {
            THRESH_BINARY_INV
        } else {
            THRESH_BINARY
        };
        opencv::imgproc::threshold(
            &greyscale,
            &mut thresholded,
            settings.col_thresh,
            255.0,
            typ,
        )
        .expect("Failed to apply threshold");

        let count = opencv::core::count_non_zero(&thresholded).unwrap();
        let met = count > settings.num_thresh;
        log::info!("got {} / {}", count, settings.num_thresh);
        // for i in 150..250 {
        //    let mut thresholded = Mat::default();

        //    opencv::imgproc::threshold(
        //        &greyscale,
        //        &mut thresholded,
        //        i.into(),
        //        255.0,
        //        typ,
        //    )
        //    .expect("Failed to apply threshold");

        //    let count = opencv::core::count_non_zero(&thresholded).unwrap();
        //    log::info!("at {} got {}", i, count);
        // }

        ProcessingResult {
            process: Processing::ColourChannelDetect(settings.clone()),
            met,
            species: 0,
            shiny: false,
        }
    }

    fn channel_detect(
        &mut self,
        settings: &ChannelDetectSettings,
        frame: &Mat,
    ) -> ProcessingResult {
        let region = frame
            .roi(opencv::core::Rect::new(
                settings.x, settings.y, settings.w, settings.h,
            ))
            .expect("Failed to crop to region of interest")
            .clone_pointee();
        let mut hsv = Mat::default();
        compat::cvt_color(&region, &mut hsv, opencv::imgproc::COLOR_BGR2HSV, 0)
            .expect("Failed to convert colour");
        let mut thresholded = Mat::default();
        let lower = Vector::from_slice(&[settings.h_lo, settings.s_lo, settings.v_lo]);
        let upper = Vector::from_slice(&[settings.h_hi, settings.s_hi, settings.v_hi]);
        opencv::core::in_range(&hsv, &lower, &upper, &mut thresholded)
            .expect("Failed to apply range");

        let count = opencv::core::count_non_zero(&thresholded).expect("Failed to count");
        let met = count > settings.num_thresh;

        ProcessingResult {
            process: Processing::ChannelDetect(settings.clone()),
            met,
            species: 0,
            shiny: false,
        }
    }

    fn set_found(&mut self, frame: &Mat, top: bool) -> ProcessingResult {
        Self::show_window(Self::FOUND_WIN, &frame);
        Self::transform_window(Self::FOUND_WIN);

        // Save to encoded frame
        opencv::imgcodecs::imencode(".png", &frame, &mut self.found, &Vector::new())
            .expect("Failed to encode frame");
        self.found_updated = true;

        if !self.found_mat.empty() {
            Self::show_window(Self::FOUND_LAST_WIN, &self.found_mat);
            Self::transform_window(Self::FOUND_LAST_WIN);
        }

        self.found_mat = frame.clone();

        ProcessingResult {
            process: Processing::SetFound(top),
            met: true,
            species: 0,
            shiny: false,
        }
    }

    fn process(&mut self, process: &Processing, frame: &Mat) -> ProcessingResult {
        match process {
            Processing::Sprite(game, species_list, flipped) => {
                self.match_sprite(game, species_list, flipped, frame, 0.0)
            }
            Processing::SpriteT(game, species_list, flipped, threshold) => {
                self.match_sprite(game, species_list, flipped, frame, *threshold)
            }
            Processing::ChannelDetect(settings) => self.channel_detect(settings, frame),
            Processing::RegionDetect(settings) => self.region_detect(settings, frame),
            Processing::ColourChannelDetect(settings) => {
                self.colour_channel_detect(settings, frame)
            }
            Processing::SetFound(top) => self.set_found(frame, *top),
            Processing::USUMShinyStar(_) => panic!("USUM Shiny Star incompatible with DS"),
            Processing::USUMBottomScreen(_) => panic!("USUM Bottom screen incompatible with DS"),
            Processing::USUMBottomScreenInv(_) => panic!("USUM Bottom screen incompatible with DS"),
            Processing::ColourChannelDetect3DS(_) => {
                panic!("ColourChannelDetect3DS incompatible with DS")
            }
            Processing::Sprite3DS(..) => panic!("Sprite3DS incompatible with DS"),
        }
    }
}
