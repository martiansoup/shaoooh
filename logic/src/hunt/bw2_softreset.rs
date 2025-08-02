use std::time::{Duration, SystemTime};

use rand::Rng;

use crate::app::states::{Game, RequestTransition, Transition};
use crate::control::{Button, ShaooohControl};
use crate::hunt::{BaseHunt, HuntFSM, HuntResult};
use crate::vision::{Processing, ProcessingResult};

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum BW2SoftResetState {
    Init,
    SoftReset,
    MainMenu1,
    MainMenu2,
    MainMenu3,
    MainMenu4,
    SelectFile,
    CGearNo1,
    CGearNo2,
    PressRight,
    Shuaaan1,
    Shuaaan2,
    StartEncounter,
    WaitInfoBar,
    Detect,
    Done,
    Wait(Duration, Box<BW2SoftResetState>),
}

const SHINY_DURATION: Duration = Duration::from_millis(6000); // TODO timed for Latios

pub(crate) struct BW2SoftReset {
    pub(crate) base: BaseHunt,
    pub(crate) state: BW2SoftResetState,
    pub(crate) timer: SystemTime,
    pub(crate) last_timer_duration: Duration,
    pub(crate) last_dbg_string: String,
    //pub(crate) stats_file: File,
}

impl BW2SoftReset {
    fn create_wait_msecs(&mut self, d: u64, state: BW2SoftResetState) -> BW2SoftResetState {
        self.base.wait_start = SystemTime::now();
        BW2SoftResetState::Wait(Duration::from_millis(d), Box::new(state))
    }
}

impl HuntFSM for BW2SoftReset {
    fn processing(&self) -> Vec<Processing> {
        if self.state == BW2SoftResetState::Detect {
            vec![Processing::Sprite(
                Game::Black2White2,
                vec![self.base.target],
                false,
            )]
        } else if self.state == BW2SoftResetState::WaitInfoBar
            || self.state == BW2SoftResetState::StartEncounter
        {
            vec![
                Processing::BW2_BAR_PRESENT,
                Processing::BW2_BLACK_SCREEN,
                Processing::BW2_WHITE_SCREEN,
                Processing::BW2_BAR_NEGATE_CONFIRM,
            ]
        } else {
            Vec::new()
        }
    }

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult {
        let incr_encounters = self.state == BW2SoftResetState::Detect;
        let mut transition = None;
        let mut detect_result = None;
        let mut bar_present = false;
        let mut bar_negate_confirm = false;
        let mut black_screen = false;
        let mut white_screen = false;

        for r in results {
            match r.process {
                Processing::Sprite(_, _, _) => detect_result = Some(r),
                Processing::BW2_BAR_PRESENT => bar_present = r.met,
                Processing::BW2_BAR_NEGATE_CONFIRM => bar_negate_confirm = r.met,
                Processing::BW2_BLACK_SCREEN => black_screen = r.met,
                Processing::BW2_WHITE_SCREEN => white_screen = r.met,
                _ => {}
            }
        }

        let seen_real_bar = bar_present & !bar_negate_confirm;

        if self.state == BW2SoftResetState::StartEncounter
            || self.state == BW2SoftResetState::WaitInfoBar
        {
            let dbg_str = format!(
                "bar={},nBar={},black={},white={}",
                bar_present, bar_negate_confirm, black_screen, white_screen
            );
            if dbg_str != self.last_dbg_string {
                log::info!("{}", dbg_str);
                self.last_dbg_string = dbg_str;
            }
        }

        let old_state = self.state.clone();

        self.state = match &self.state {
            BW2SoftResetState::Init => BW2SoftResetState::SoftReset,
            BW2SoftResetState::SoftReset => {
                control.gen5_soft_reset();
                let mut rng = rand::rng();
                let delay = 5000 + rng.random_range(0..1000);
                self.create_wait_msecs(delay, BW2SoftResetState::MainMenu1)
            }
            BW2SoftResetState::MainMenu1 => {
                control.press(Button::A);
                let delay = 5000;
                self.create_wait_msecs(delay, BW2SoftResetState::MainMenu2)
            }
            BW2SoftResetState::MainMenu2 => {
                control.press(Button::A);
                let delay = 5000;
                self.create_wait_msecs(delay, BW2SoftResetState::MainMenu3)
            }
            BW2SoftResetState::MainMenu3 => {
                control.press(Button::A);
                let delay = 7000;
                self.create_wait_msecs(delay, BW2SoftResetState::MainMenu4)
            }
            BW2SoftResetState::MainMenu4 => {
                control.press(Button::A);
                let delay = 3500;
                self.create_wait_msecs(delay, BW2SoftResetState::SelectFile)
            }
            BW2SoftResetState::SelectFile => {
                control.press(Button::A);
                let delay = 2500;
                self.create_wait_msecs(delay, BW2SoftResetState::CGearNo1)
            }
            BW2SoftResetState::CGearNo1 => {
                control.press(Button::B);
                let delay = 1000;
                self.create_wait_msecs(delay, BW2SoftResetState::CGearNo2)
            }
            BW2SoftResetState::CGearNo2 => {
                control.press(Button::A);
                let mut rng = rand::rng();
                let delay = 8000 + rng.random_range(0..1000);
                self.create_wait_msecs(delay, BW2SoftResetState::PressRight)
            }
            BW2SoftResetState::PressRight => {
                control.press(Button::Right);
                let delay = 2000;
                self.create_wait_msecs(delay, BW2SoftResetState::Shuaaan1)
            }
            BW2SoftResetState::Shuaaan1 => {
                control.press(Button::A);
                let delay = 3500;
                self.create_wait_msecs(delay, BW2SoftResetState::Shuaaan2)
            }
            BW2SoftResetState::Shuaaan2 => {
                control.press(Button::A);
                self.timer = SystemTime::now();
                BW2SoftResetState::StartEncounter
            }
            BW2SoftResetState::StartEncounter => {
                if white_screen {
                    BW2SoftResetState::WaitInfoBar
                } else {
                    BW2SoftResetState::StartEncounter
                }
            }
            BW2SoftResetState::WaitInfoBar => {
                if seen_real_bar {
                    self.last_timer_duration = self.timer.elapsed().unwrap();
                    BW2SoftResetState::Detect
                } else {
                    BW2SoftResetState::WaitInfoBar
                }
            }
            BW2SoftResetState::Detect => {
                if let Some(detect) = detect_result {
                    log::info!(
                        "sprite = {}, duration = {:?}",
                        detect.shiny,
                        self.last_timer_duration
                    );
                    if detect.shiny || (self.last_timer_duration > SHINY_DURATION) {
                        transition = Some(RequestTransition {
                            transition: Transition::FoundTarget,
                            arg: None,
                        });
                        BW2SoftResetState::Done
                    } else {
                        let mut rng = rand::rng();
                        let delay = 500 + rng.random_range(0..2000);
                        self.create_wait_msecs(delay, BW2SoftResetState::SoftReset)
                    }
                } else {
                    log::error!("No detect result found");
                    BW2SoftResetState::Done
                }
            }
            BW2SoftResetState::Done => BW2SoftResetState::Done,
            BW2SoftResetState::Wait(duration, next) => {
                if self.base.wait_start.elapsed().expect("Failed to get time") > *duration {
                    (**next).clone()
                } else {
                    self.state.clone()
                }
            }
        };

        // TODO don't print waits
        if old_state != self.state {
            log::debug!("STATE = {:?} -> {:?}", old_state, self.state);
        }

        HuntResult {
            transition,
            incr_encounters,
        }
    }

    fn cleanup(&mut self) {
        log::info!("Closing FSM");
    }
}
