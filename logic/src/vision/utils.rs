use std::collections::HashMap;

use opencv::prelude::*;

#[expect(dead_code, reason = "TODO")]
pub struct VisionUtils {
    // Reference, Shiny, Mask
    reference: HashMap<u32, (Mat, Mat, Mat)>,
}

impl VisionUtils {}

impl Default for VisionUtils {
    fn default() -> Self {
        Self {
            reference: Default::default(),
        }
    }
}
