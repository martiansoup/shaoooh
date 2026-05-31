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
            todo!("Request input");
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

    fn read_found(&mut self) -> &[u8] {
        &self.frame
    }

    fn new_found(&self) -> bool {
        false
    }
}

impl NopVision {
    pub fn new() -> Self {
        NopVision {
            frame: std::fs::read("static/metamon.png").unwrap_or_default(),
        }
    }
}

impl Default for NopVision {
    fn default() -> Self {
        NopVision::new()
    }
}
