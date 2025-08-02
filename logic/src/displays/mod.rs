use tokio::sync::watch;

use crate::app::states::AppState;

mod gfx;
mod lights;

pub use lights::LightsDisplay;

pub struct DisplayWrapper {
    func: fn() -> Box<dyn StateReceiver>,
    name: String,
}

impl DisplayWrapper {
    pub fn new(name: String, func: fn() -> Box<dyn StateReceiver>) -> Self {
        DisplayWrapper { func, name }
    }
    pub fn thread(&mut self, mut rx: watch::Receiver<AppState>) {
        let mut inner = (self.func)();
        while rx.has_changed().is_ok() {
            let state_copy = { (*rx.borrow_and_update()).clone() };
            inner.display(state_copy);
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

pub trait StateReceiver {
    fn display(&mut self, state: AppState);
}
