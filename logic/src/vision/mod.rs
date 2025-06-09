use opencv::{core::Vector, highgui, prelude::*, videoio::VideoCapture};

pub enum Processing {
    Gen3TextBox,
}

pub struct ProcessingResult {
    process: Processing,
    met: bool,
    species: u32,
    shiny: bool,
}

pub struct Vision {
    cam: VideoCapture,
    encoded: Vector<u8>,
}

impl Vision {
    const WIDTH: u32 = 320;
    const HEIGHT: u32 = 240;
    const BORDER_KEEP: u32 = 0;
    const BORDER_LR: u32 = 16;
    const BORDER_TB: u32 = 20;
    const X0: u32 = Vision::BORDER_LR - Vision::BORDER_KEEP;
    const Y0: u32 = Vision::BORDER_TB - Vision::BORDER_KEEP;
    const X1: u32 = Vision::WIDTH - (Vision::BORDER_LR - Vision::BORDER_KEEP);
    const Y1: u32 = Vision::HEIGHT - (Vision::BORDER_TB - Vision::BORDER_KEEP);

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

    pub fn process_next_frame(&mut self, processing: Vec<Processing>) -> Vec<ProcessingResult> {
        let mut frame = Mat::default();
        self.cam.read(&mut frame).expect("Failed to read frame");

        // Save to encoded frame
        opencv::imgcodecs::imencode(".png", &frame, &mut self.encoded, &Vector::new())
            .expect("Failed to encode frame");

        // TODO Do per-frame processing
        // TODO don't show gui ?
        highgui::imshow("capture", &frame).expect("Failed to show capture");
        highgui::wait_key(1).expect("Event loop failed");
        Vec::new()
    }

    pub fn read_frame(&self) -> &[u8] {
        self.encoded.as_slice()
    }
}
