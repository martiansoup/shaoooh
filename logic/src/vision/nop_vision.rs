use crate::vision::{BotVision, ProcessingResult};

pub struct NopVision {
    frame: Vec<u8>,
}

impl BotVision for NopVision {
    fn process_next_frame(
        &mut self,
        processing: &[super::Processing],
    ) -> Option<Vec<ProcessingResult>> {
        let mut results = vec![];
        for proc in processing {
            results.push(ProcessingResult {
                process: proc.clone(),
                met: true,
                species: 0,
                shiny: true,
            });
        }

        opencv::highgui::wait_key(1).expect("Event loop failed");

        Some(results)
    }

    fn read_frame(&self) -> &[u8] {
        &self.frame
    }

    fn read_frame2(&self) -> &[u8] {
        &self.frame
    }
}

impl NopVision {
    pub fn new() -> Self {
        let frame = if let Ok(f) = std::fs::read("static/metamon.png") {
            f
        } else {
            vec![]
        };

        NopVision { frame }
    }
}

impl Default for NopVision {
    fn default() -> Self {
        NopVision::new()
    }
}
