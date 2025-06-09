use std::time::{Duration, SystemTime, SystemTimeError};

use opencv::ccalib::RECTIFY_CYLINDRICAL;

use crate::{
    app::states::{Game, Method, RequestTransition, Transition},
    control::{Button, ShaooohControl},
    vision::{Processing, ProcessingResult},
};

pub struct HuntResult {
    pub(crate) transition: Option<RequestTransition>,
    pub(crate) incr_encounters: bool,
}

pub trait HuntFSM {
    // TODO methods to be able to draw FSM status

    fn processing(&self) -> Vec<Processing>;

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult;

    fn cleanup(&mut self) {}
}

pub struct HuntBuild {}

impl HuntBuild {
    pub fn build(target: u32, game: Game, method: Method) -> Result<impl HuntFSM, ()> {
        let base = BaseHunt {
            target,
            game: game.clone(),
            method: method.clone(),
            wait_start: SystemTime::now()
        };
        if game == Game::FireRedLeafGreen && method == Method::SoftResetGift && (target == 1 || target == 4 || target == 7) {
            Ok(FRLGStarterGift { base, state: FRLGStarterGiftState::SoftReset,  })
        } else {
            log::error!(
                "Hunt not found for target:{}, game:{:?}, method:{:?}",
                target,
                game,
                method
            );
            Err(())
        }
    }
}

struct BaseHunt {
    target: u32,
    game: Game,
    method: Method,
    wait_start: SystemTime
}

#[derive(PartialEq, Clone, Debug)]
enum FRLGStarterGiftState {
    SoftReset,
    StartToGetToTitle,
    StartToSelectFile,
    AToContinue,
    BToSkipReplay,
    AToGetStarter,
    AToAdvText1,
    AToSelectYes,
    AToAdvText2,
    BToNotName,
    AToAdvText3,
    StartToMenu,
    AToSelParty,
    AToSelStarter,
    AToSelSummary,
    Detect,
    Done,
    Wait(Duration, Box<FRLGStarterGiftState>)
}

struct FRLGStarterGift {
    base: BaseHunt,
    state: FRLGStarterGiftState
}

impl FRLGStarterGift {
    fn create_wait_secs(&mut self, d: u64, state: FRLGStarterGiftState) -> FRLGStarterGiftState {
        self.base.wait_start = SystemTime::now();
        FRLGStarterGiftState::Wait(Duration::from_secs(d), Box::new(state))
    }
}

impl HuntFSM for FRLGStarterGift {
    fn processing(&self) -> Vec<Processing> {
        if self.state == FRLGStarterGiftState::Detect {
            vec![Processing::Sprite(Game::FireRedLeafGreen, vec![self.base.target], true)]
        } else {
            Vec::new()
        }
    }


    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult {
        let incr_encounters = self.state == FRLGStarterGiftState::Detect;

        let found_shiny = results.iter().any(|r| r.shiny);
        let mut transition = None;

        match &self.state {
            FRLGStarterGiftState::Wait(_, _) => {},
            s => {
                log::debug!("STATE = {:?}", s);
            }
        }

        self.state = match &self.state {
            FRLGStarterGiftState::SoftReset => {
                control.gen3_soft_reset();
                self.create_wait_secs(5, FRLGStarterGiftState::StartToGetToTitle)
            }
            FRLGStarterGiftState::StartToGetToTitle => {
                control.press(Button::Start);
                self.create_wait_secs(6, FRLGStarterGiftState::StartToSelectFile)
            }
            FRLGStarterGiftState::StartToSelectFile => {
                control.press(Button::Start);
                self.create_wait_secs(6, FRLGStarterGiftState::AToContinue)
            }
            FRLGStarterGiftState::AToContinue => {
                control.press(Button::A);
                self.create_wait_secs(3, FRLGStarterGiftState::BToSkipReplay)
            }
            FRLGStarterGiftState::BToSkipReplay => {
                control.press(Button::B);
                self.create_wait_secs(3, FRLGStarterGiftState::AToGetStarter)
            }
            FRLGStarterGiftState::AToGetStarter => {
                control.press(Button::A);
                self.create_wait_secs(2, FRLGStarterGiftState::AToAdvText1)
            }
            FRLGStarterGiftState::AToAdvText1 => {
                control.press(Button::A);
                self.create_wait_secs(1, FRLGStarterGiftState::AToSelectYes)
            }
            FRLGStarterGiftState::AToSelectYes => {
                control.press(Button::A);
                self.create_wait_secs(1, FRLGStarterGiftState::AToAdvText2)
            }
            FRLGStarterGiftState::AToAdvText2 => {
                control.press(Button::A);
                self.create_wait_secs(5, FRLGStarterGiftState::BToNotName)
            }
            FRLGStarterGiftState::BToNotName => {
                control.press(Button::B);
                self.create_wait_secs(5, FRLGStarterGiftState::AToAdvText3)
            }
            FRLGStarterGiftState::AToAdvText3 => {
                control.press(Button::A);
                self.create_wait_secs(5, FRLGStarterGiftState::StartToMenu)
            }
            FRLGStarterGiftState::StartToMenu => {
                control.press(Button::Start);
                self.create_wait_secs(1, FRLGStarterGiftState::AToSelParty)
            }
            FRLGStarterGiftState::AToSelParty => {
                control.press(Button::A);
                self.create_wait_secs(1, FRLGStarterGiftState::AToSelStarter)
            }
            FRLGStarterGiftState::AToSelStarter => {
                control.press(Button::A);
                self.create_wait_secs(1, FRLGStarterGiftState::AToSelSummary)
            }
            FRLGStarterGiftState::AToSelSummary => {
                control.press(Button::A);
                self.create_wait_secs(3, FRLGStarterGiftState::Detect)
            }
            FRLGStarterGiftState::Detect => {
                if (found_shiny) {
                    transition = Some(RequestTransition { transition: Transition::FoundTarget, arg: None });
                    FRLGStarterGiftState::Done
                } else {
                    FRLGStarterGiftState::SoftReset
                }
            }
            FRLGStarterGiftState::Done => FRLGStarterGiftState::Done,
            FRLGStarterGiftState::Wait(duration, next) => {
                if self.base.wait_start.elapsed().expect("Failed to get time") > *duration {
                    (**next).clone()
                } else {
                    self.state.clone()
                }
            }
        };

        HuntResult {
            transition,
            incr_encounters,
        }
    }

    fn cleanup(&mut self) {
        log::info!("Closing FSM");
    }
}
