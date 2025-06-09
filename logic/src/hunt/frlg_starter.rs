use std::time::{Duration, SystemTime};
use crate::{app::states::{Game, RequestTransition, Transition}, control::{Button, ShaooohControl}, hunt::{BaseHunt, HuntFSM, HuntResult}, vision::{Processing, ProcessingResult}};

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum FRLGStarterGiftState {
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

pub(crate) struct FRLGStarterGift {
    pub(crate) base: BaseHunt,
    pub(crate) state: FRLGStarterGiftState
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
                if found_shiny {
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
