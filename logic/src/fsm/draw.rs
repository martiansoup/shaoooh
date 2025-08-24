use opencv::{
    core::{Mat, Rect, VecN},
    imgcodecs::IMREAD_COLOR,
    imgproc::LINE_8,
};

pub struct Graph {
    base: Mat,
    x: Vec<i32>,
    y: Vec<i32>,
    widths: Vec<i32>,
    heights: Vec<i32>,
}

impl Graph {
    pub fn new(fname: &str, x: Vec<i32>, y: Vec<i32>, w: Vec<i32>, h: Vec<i32>) -> Self {
        let mat = opencv::imgcodecs::imread(fname, IMREAD_COLOR).expect("Failed to read graph");
        Self {
            base: mat,
            x,
            y,
            widths: w,
            heights: h,
        }
    }

    pub fn with_state(&self, state: usize) -> Mat {
        let mut with_highlight = self.base.clone();
        let x = self.x[state];
        let y = self.y[state];
        let width = self.widths[state];
        let height = self.heights[state];

        let rect = Rect {
            x,
            y,
            width,
            height,
        };

        //let colour : VecN<f64, 4> = VecN([166.0, 207.0, 160.0, 0.0]);
        let colour: VecN<f64, 4> = VecN([94.0, 124.0, 39.0, 0.0]);

        opencv::imgproc::rectangle(&mut with_highlight, rect, colour, 5, LINE_8, 0)
            .expect("Failed to draw rectangle");
        // opencv::imgproc::rectangle(&mut with_highlight, rect, 0.0.into(), 5, LINE_8, 0)
        //     .expect("Failed to draw rectangle");

        with_highlight
    }
}
