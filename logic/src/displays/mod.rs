use tokio::sync::watch;

use crate::app::states::AppState;

mod display;
mod gfx;
mod webhook;

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
mod lights;

pub use display::ScreenDisplay;
pub use gfx::GfxDisplay;
pub use webhook::Webhook;

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
pub use lights::LightsDisplay;

pub struct DisplayWrapper {
    func: Option<Box<dyn FnOnce() -> Box<dyn StateReceiver> + Send>>,
    name: String,
}

impl DisplayWrapper {
    pub fn new(name: String, func: Box<dyn FnOnce() -> Box<dyn StateReceiver> + Send>) -> Self {
        DisplayWrapper {
            func: Some(func),
            name,
        }
    }
    pub fn thread(&mut self, mut rx: watch::Receiver<AppState>) {
        if let Some(func) = self.func.take() {
            let mut inner = func();
            if inner.always_run() {
                while rx.has_changed().is_ok() {
                    let state_copy = { rx.borrow().clone() };
                    inner.display(state_copy);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            } else {
                while rx.has_changed().is_ok() {
                    let state_copy = { (*rx.borrow_and_update()).clone() };
                    inner.display(state_copy);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            inner.cleanup();
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

pub trait StateReceiver {
    fn display(&mut self, state: AppState);
    fn cleanup(&mut self);
    fn always_run(&self) -> bool {
        false
    }
}
