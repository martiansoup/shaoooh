use std::time::{Duration, SystemTime};

use opencv::highgui::{WINDOW_AUTOSIZE, WINDOW_GUI_NORMAL, WINDOW_KEEPRATIO};

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::{
    control::{BotControl, Button, Delay},
    fsm::StateMachine,
    hunt::HuntResult,
    vision::{Processing, ProcessingResult},
};

#[derive(Debug, Clone)]
pub struct HuntStateOutput {
    pub button: Button,
    pub delay: Delay,
}

impl HuntStateOutput {
    pub fn new(button: Button, delay: Delay) -> Self {
        HuntStateOutput { button, delay }
    }

    pub fn button(button: Button) -> Self {
        HuntStateOutput {
            button,
            delay: Delay::Tenth,
        }
    }
}

#[derive(Debug)]
pub struct InternalHuntState {
    pub toggle: bool,
    pub atomic: Arc<AtomicBool>,
    pub time: SystemTime,
    pub last_duration: Duration,
    pub counter: usize,
}

impl InternalHuntState {
    pub fn new(atomic: Arc<AtomicBool>) -> Self {
        Self {
            toggle: Default::default(),
            atomic,
            time: SystemTime::now(),
            last_duration: Duration::default(),
            counter: 0,
        }
    }
}

#[derive(Debug)]
pub struct HuntFSM {
    fsm: StateMachine<Processing, ProcessingResult, HuntStateOutput, HuntResult, InternalHuntState>,
}

impl HuntFSM {
    pub fn new(
        mut fsm: StateMachine<
            Processing,
            ProcessingResult,
            HuntStateOutput,
            HuntResult,
            InternalHuntState,
        >,
    ) -> Self {
        // TODO temporary file
        fsm.graph_file("current_fsm").expect("Failed to draw graph");
        opencv::highgui::named_window(
            "fsm",
            WINDOW_AUTOSIZE | WINDOW_KEEPRATIO | WINDOW_GUI_NORMAL,
        )
        .unwrap_or_else(|_| panic!("Failed to create 'fsm' window"));
        opencv::highgui::move_window("fsm", 576, 32)
            .unwrap_or_else(|_| panic!("Failed to move 'fsm' window"));
        HuntFSM { fsm }
    }

    pub fn processing(&self) -> &Vec<Processing> {
        self.fsm.inputs()
    }

    pub fn step(
        &mut self,
        control: &mut Box<dyn BotControl>,
        results: Vec<ProcessingResult>,
    ) -> HuntResult {
        let outputs = self.fsm.outputs();
        if !outputs.is_empty() {
            if outputs.windows(2).all(|v| v[0].delay == v[1].delay) {
                // All delays match, press buttons togther
                let buttons: Vec<&Button> = outputs.iter().map(|v| &v.button).collect();
                control.presses_delay(buttons.as_slice(), &outputs[0].delay);
            } else {
                // Buttons have different delays, press in sequence
                for out in outputs {
                    control.press_delay(&out.button, &out.delay);
                }
            }
        }

        self.step_no_output(results)
    }

    pub fn cleanup(&mut self) {}

    pub fn current_name(&self) -> String {
        self.fsm.current_name()
    }

    pub fn debug_name(&self) -> String {
        self.fsm.debug_name()
    }

    // Only public for testing
    pub fn step_no_output(&mut self, results: Vec<ProcessingResult>) -> HuntResult {
        if let Some(output) = self.fsm.process(results) {
            output
        } else {
            HuntResult {
                transition: None,
                incr_encounters: false,
            }
        }
    }

    pub fn graph_file(&mut self, file_root: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.fsm.graph_file(file_root)
    }

    pub fn graph(&self) -> Option<opencv::core::Mat> {
        self.fsm.graph_with_state()
    }

    pub fn display(&self) {
        if let Some(m) = self.graph() {
            // TODO combine window drawing code with vision
            opencv::highgui::imshow("fsm", &m)
                .unwrap_or_else(|_| panic!("Failed to show 'fsm' window"));
            opencv::highgui::move_window("fsm", 632, 32)
                .unwrap_or_else(|_| panic!("Failed to move 'fsm' window"));
        }
    }
}
