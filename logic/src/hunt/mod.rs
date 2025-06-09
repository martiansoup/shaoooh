use crate::{
    app::states::{Game, Method},
    control::ShaooohControl,
    vision::{Processing, ProcessingResult},
};

pub trait HuntFSM {
    // TODO methods to be able to draw FSM status

    fn processing(&self) -> Vec<Processing>;

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>);

    fn cleanup(&mut self);
}

pub struct HuntBuild {}

impl HuntBuild {
    pub fn build(target: u32, game: Game, method: Method) -> Result<impl HuntFSM, ()> {
        let base = BaseHunt {
            target,
            game: game.clone(),
            method: method.clone(),
        };
        if game == Game::FireRedLeafGreen && method == Method::SoftResetGift {
            Ok(FRLGStarterGift { base })
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
}

struct FRLGStarterGift {
    base: BaseHunt,
}

impl HuntFSM for FRLGStarterGift {
    fn processing(&self) -> Vec<Processing> {
        Vec::new()
    }

    fn step(&mut self, control: &mut ShaooohControl, results: Vec<ProcessingResult>) {
        log::info!("Stepping FSM");
    }

    fn cleanup(&mut self) {
        log::info!("Closing FSM");
    }
}
