use crate::{
    app::states::{Game, RequestTransition, Transition},
    control::{Button, ShaooohControl},
    hunt::{BaseHunt, HuntFSM, HuntResult},
    vision::{Processing, ProcessingResult},
};
use std::time::{Duration, SystemTime};

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
    RToSelStats,
    Detect,
    Done,
    Wait(Duration, Box<FRLGStarterGiftState>),
}

pub(crate) struct FRLGStarterGift {
    pub(crate) base: BaseHunt,
    pub(crate) state: FRLGStarterGiftState,
}

impl FRLGStarterGift {
    fn create_wait_secs(&mut self, d: u64, state: FRLGStarterGiftState) -> FRLGStarterGiftState {
        self.base.wait_start = SystemTime::now();
        FRLGStarterGiftState::Wait(Duration::from_secs(d), Box::new(state))
    }

    fn create_wait_msecs(&mut self, d: u64, state: FRLGStarterGiftState) -> FRLGStarterGiftState {
        self.base.wait_start = SystemTime::now();
        FRLGStarterGiftState::Wait(Duration::from_millis(d), Box::new(state))
    }
}

impl HuntFSM for FRLGStarterGift {
    fn processing(&self) -> Vec<Processing> {
        if self.state == FRLGStarterGiftState::Detect {
            vec![
                Processing::Sprite(Game::FireRedLeafGreen, vec![self.base.target], true),
                Processing::FRLGShinyStar,
            ]
        } else {
            Vec::new()
        }
    }

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) -> HuntResult {
        let incr_encounters = self.state == FRLGStarterGiftState::Detect;
        let mut shiny_sprite = false;
        let mut shiny_star = false;

        for r in results {
            match r.process {
                Processing::Sprite(_, _, _) => shiny_sprite = r.shiny,
                Processing::FRLGShinyStar => shiny_star = r.shiny,
                _ => {}
            }
        }

        let found_shiny = shiny_sprite | shiny_star;
        if found_shiny {
            log::info!("shiny_sprite = {}, shiny_star = {}", shiny_sprite, shiny_star);
        }
        let mut transition = None;

        match &self.state {
            FRLGStarterGiftState::Wait(_, _) => {}
            FRLGStarterGiftState::Done => {}
            s => {
                log::debug!("STATE = {:?}", s);
            }
        }

        self.state = match &self.state {
            FRLGStarterGiftState::SoftReset => {
                control.gen3_soft_reset();
                self.create_wait_msecs(3500, FRLGStarterGiftState::StartToGetToTitle)
            }
            FRLGStarterGiftState::StartToGetToTitle => {
                control.press(Button::Start);
                self.create_wait_msecs(4500, FRLGStarterGiftState::StartToSelectFile)
            }
            FRLGStarterGiftState::StartToSelectFile => {
                control.press(Button::Start);
                self.create_wait_msecs(3000, FRLGStarterGiftState::AToContinue)
            }
            FRLGStarterGiftState::AToContinue => {
                control.press(Button::A);
                self.create_wait_msecs(1500, FRLGStarterGiftState::BToSkipReplay)
            }
            FRLGStarterGiftState::BToSkipReplay => {
                control.press(Button::B);
                self.create_wait_msecs(2500, FRLGStarterGiftState::AToGetStarter)
            }
            FRLGStarterGiftState::AToGetStarter => {
                control.press(Button::A);
                self.create_wait_msecs(1000, FRLGStarterGiftState::AToAdvText1)
            }
            FRLGStarterGiftState::AToAdvText1 => {
                control.press(Button::A);
                self.create_wait_msecs(1000, FRLGStarterGiftState::AToSelectYes)
            }
            FRLGStarterGiftState::AToSelectYes => {
                control.press(Button::A);
                self.create_wait_msecs(500, FRLGStarterGiftState::AToAdvText2)
            }
            FRLGStarterGiftState::AToAdvText2 => {
                control.press(Button::A);
                self.create_wait_msecs(4500, FRLGStarterGiftState::BToNotName)
            }
            FRLGStarterGiftState::BToNotName => {
                control.press(Button::B);
                self.create_wait_msecs(2500, FRLGStarterGiftState::AToAdvText3)
            }
            FRLGStarterGiftState::AToAdvText3 => {
                control.press(Button::A);
                self.create_wait_msecs(3750, FRLGStarterGiftState::StartToMenu)
            }
            FRLGStarterGiftState::StartToMenu => {
                control.press(Button::Start);
                self.create_wait_msecs(1000, FRLGStarterGiftState::AToSelParty)
            }
            FRLGStarterGiftState::AToSelParty => {
                control.press(Button::A);
                self.create_wait_msecs(1000, FRLGStarterGiftState::AToSelStarter)
            }
            FRLGStarterGiftState::AToSelStarter => {
                control.press(Button::A);
                self.create_wait_msecs(1000, FRLGStarterGiftState::AToSelSummary)
            }
            FRLGStarterGiftState::AToSelSummary => {
                control.press(Button::A);
                self.create_wait_msecs(1000, FRLGStarterGiftState::RToSelStats)
            }
            FRLGStarterGiftState::RToSelStats => {
                control.press(Button::Right);
                self.create_wait_msecs(1000, FRLGStarterGiftState::Detect)
            }
            FRLGStarterGiftState::Detect => {
                if found_shiny {
                    transition = Some(RequestTransition {
                        transition: Transition::FoundTarget,
                        arg: None,
                    });
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
