use crate::{
    app::states::{AppState, HuntState},
    lights::{Lights, PixelData},
};
use std::thread;
use std::time::Duration;

pub struct LightsDisplay {
    anim: u64,
    lights: Lights,
}

impl LightsDisplay {
    const NUM_PIXELS: u32 = 7;
}

impl super::StateReceiver for LightsDisplay {
    fn display(&mut self, state: AppState) {
        let mut data = Vec::new();

        let interesting_state =
            (state.state != HuntState::Idle) && (state.state != HuntState::Hunt);

        let num: u64 = Self::NUM_PIXELS.into();
        let num_circle = num - 1;
        if interesting_state {
            let r = 0;
            let (g, b) = if state.state == HuntState::FoundTarget {
                (50, 0)
            } else {
                (0, 50)
            };
            let w = 0;
            data.push(PixelData { r, g, b, w });
            for n in 1..num {
                let highlight_pixel = (self.anim + num_circle) % num_circle == (n - 1);
                let highlight_pixel_m1 = (self.anim + num_circle - 1) % num_circle == (n - 1);
                let highlight_pixel_m2 = (self.anim + num_circle - 2) % num_circle == (n - 1);
                let c = if highlight_pixel {
                    60
                } else if highlight_pixel_m1 {
                    10
                } else if highlight_pixel_m2 {
                    5
                } else {
                    0
                };
                let r = 0;
                let (g, b) = if state.state == HuntState::FoundTarget {
                    (c, 0)
                } else {
                    (0, c)
                };
                let w = 0;
                data.push(PixelData { r, g, b, w });
            }
            self.anim += 1;
        } else if state.state == HuntState::Idle {
            for _n in 0..num {
                data.push(PixelData {
                    r: 0,
                    g: 0,
                    b: 0,
                    w: 0,
                });
            }
        } else {
            data.push(PixelData {
                r: 0,
                g: 0,
                b: 0,
                w: 0,
            });
            for n in 1..num {
                let highlight_pixel = (state.encounters + num_circle) % num_circle == (n - 1);
                let highlight_pixel_m1 =
                    (state.encounters + num_circle - 1) % num_circle == (n - 1);
                let highlight_pixel_m2 =
                    (state.encounters + num_circle - 2) % num_circle == (n - 1);
                let r = if highlight_pixel {
                    60
                } else if highlight_pixel_m1 {
                    25
                } else if highlight_pixel_m2 {
                    10
                } else {
                    0
                };
                let g = 0;
                let b = 0;
                let w = 0;
                data.push(PixelData { r, g, b, w });
            }
        }

        self.lights.draw(data);
        thread::sleep(Duration::from_millis(100));
    }
}

impl Default for LightsDisplay {
    fn default() -> Self {
        let anim = 0;
        let lights = Lights::new(Self::NUM_PIXELS, 18);
        Self { anim, lights }
    }
}
