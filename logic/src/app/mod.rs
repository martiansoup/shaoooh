use std::{
    fs::read_to_string,
    io::{BufWriter, Write},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serialport::SerialPort;
use tokio::sync::{mpsc, oneshot, watch};
use tower_http::services::{ServeDir, ServeFile};
pub(crate) mod states;
use crate::{
    control::{Button, ShaooohControl},
    hunt::{HuntBuild, HuntFSM},
    lights::{Lights, PixelData},
    vision::Vision,
};
use states::*;
use tokio::signal;

// Response to any requests for the current state, also includes possible transitions
#[derive(Clone, Serialize)]
struct ResponseAppState {
    state: AppState,
    transitions: Vec<StateTransition>,
}

#[derive(Clone)]
struct ApiState {
    rx: watch::Receiver<AppState>,
    tx: mpsc::Sender<RequestTransition>,
    button_tx: mpsc::Sender<Button>,
    image: Arc<Mutex<Vec<u8>>>,
}

pub struct Shaoooh {
    api: Option<ApiState>,
    app: AppState,
    tx: watch::Sender<AppState>,
    rx: mpsc::Receiver<RequestTransition>,
    button_rx: mpsc::Receiver<Button>,
    image: Arc<Mutex<Vec<u8>>>,
    // TODO split to separate module - have StateConsumer list?
    serial_disp: Option<Box<dyn SerialPort>>,
}

// Struct to load/save from disc
#[derive(Clone, Serialize, Deserialize)]
struct HuntInformation {
    name: String,
    species: u32,
    game: Game,
    method: Method,
    encounters: u64,
    phases: Vec<Phase>,
    complete: bool,
}

#[derive(Deserialize)]
struct UserConfig {
    api_key: Option<String>,
    user_id: Option<String>,
}

impl Shaoooh {
    pub const VIDEO_NUM: u32 = 0;
    pub const VIDEO_DEV: &str = "/dev/video0";

    pub fn new() -> Self {
        let app = AppState {
            state: HuntState::Idle,
            arg: None,
            encounters: 0,
            phases: Vec::new(),
            last_phase: 0,
        };
        let (state_tx, state_rx) = watch::channel(app.clone());
        let (transition_tx, transition_rx) = mpsc::channel(1);
        let (button_tx, button_rx) = mpsc::channel(8);
        let image_mutex = Arc::new(Mutex::new(Vec::new()));

        let api = ApiState {
            rx: state_rx,
            tx: transition_tx,
            button_tx,
            image: image_mutex.clone(),
        };
        let serial_disp = serialport::new("/dev/ttyACM0", 115200).open().ok();
        Self {
            api: Some(api),
            app,
            tx: state_tx,
            rx: transition_rx,
            button_rx,
            image: image_mutex,
            serial_disp,
        }
    }

    pub fn routes(state: ApiState) -> Router {
        let static_dir = ServeDir::new("./static");
        let index = ServeFile::new("index.html");

        Router::new()
            .route_service("/", index)
            .nest_service("/static", static_dir)
            .route("/api/state", get(get_state).post(post_state))
            .route("/api/button", post(post_button))
            .route("/api/frame", get(get_frame))
            .with_state(state)
    }

    fn filename_from_name(name: &str) -> String {
        format!("hunts/hunt_{}.json", name)
    }

    fn try_get_encounters(name: &str) -> (Vec<Phase>, u64) {
        if std::fs::exists(Self::filename_from_name(name)).unwrap_or(false) {
            // TODO error check information? And check if already complete?
            let data = std::fs::read_to_string(Self::filename_from_name(name))
                .expect("Couldn't read file");
            let hunt: HuntInformation = serde_json::from_str(&data).expect("Failed to parse json");
            (hunt.phases, hunt.encounters)
        } else {
            (Vec::new(), 0)
        }
    }

    fn update_state(&mut self) {
        if self.app.state != HuntState::Idle {
            let name = self.app.arg.as_ref().unwrap().name.clone();
            let state = HuntInformation {
                name: self.app.arg.as_ref().unwrap().name.clone(),
                species: self.app.arg.as_ref().unwrap().species,
                game: self.app.arg.as_ref().unwrap().game.clone(),
                method: self.app.arg.as_ref().unwrap().method.clone(),
                encounters: self.app.encounters,
                phases: self.app.phases.clone(),
                complete: self.app.state == HuntState::FoundTarget,
            };
            let file = std::fs::File::create(Self::filename_from_name(&name)).unwrap();
            let mut writer = BufWriter::new(file);
            serde_json::to_writer(&mut writer, &state);
            writer.flush();
            if let Some(tx) = &mut self.serial_disp {
                // TODO reset enc count globally
                let phased = self.app.encounters
                    - self
                        .app
                        .phases
                        .iter()
                        .map(|x| x.encounters)
                        .max()
                        .unwrap_or(0);
                let enc_str = format!("E{}e", phased);
                tx.write_all(enc_str.as_bytes());
            };
        }
        self.tx
            .send(self.app.clone())
            .expect("Couldn't update state");
    }

    fn transition_logic(
        &mut self,
        from: HuntState,
        hunt: &mut Option<Box<dyn HuntFSM>>,
        transition: &Transition,
    ) -> bool {
        if self.app.state == HuntState::Hunt && from != HuntState::Hunt {
            // Build hunt object
            let target = self.app.arg.as_ref().unwrap().species;
            let game = self.app.arg.as_ref().unwrap().game.clone();
            let method = self.app.arg.as_ref().unwrap().method.clone();
            let new_hunt = HuntBuild::build(target, game, method);
            if let Some(tx) = &mut self.serial_disp {
                let tgt_str = format!("T{}e", target);
                log::info!("Setting target on display to {}", tgt_str);
                tx.write_all(tgt_str.as_bytes());
            };
            match new_hunt {
                Ok(h) => *hunt = Some(h),
                Err(_) => return false,
            };
        }
        if self.app.state != HuntState::FoundNonTarget
            && from == HuntState::FoundNonTarget
            && *transition != Transition::FalseDetect
        {
            let phase = Phase {
                caught: *transition == Transition::Caught,
                species: self.app.last_phase,
                encounters: self.app.encounters,
            };
            self.app.phases.push(phase);
        }
        if self.app.state == HuntState::Idle {
            if let Some(h) = hunt {
                h.cleanup();
            }
            *hunt = None;
        }
        true
    }

    fn do_transition(
        &mut self,
        transition_req: RequestTransition,
        hunt: &mut Option<Box<dyn HuntFSM>>,
        automatic: bool,
    ) {
        let possible_transitions = self.app.state.possible_transitions();
        let try_transition = possible_transitions
            .iter()
            .find(|x| x.transition == transition_req.transition);
        match try_transition {
            Some(transition) => {
                log::info!("Got transition {:?}", transition);
                let arg = transition_req.arg;
                if arg.is_some() == transition.needs_arg && (automatic || !transition.automatic) {
                    let prev_state = self.app.state.clone();
                    self.app.state = transition.next_state.clone();
                    if transition.needs_arg {
                        if transition.next_state == HuntState::FoundNonTarget {
                            self.app.last_phase = arg.unwrap().species;
                        } else {
                            self.app.arg = Some(arg.unwrap());
                            (self.app.phases, self.app.encounters) =
                                Self::try_get_encounters(&self.app.arg.as_ref().unwrap().name);
                            log::info!("Got argument: {:?}", self.app.arg);
                        }
                    }
                    if self.transition_logic(prev_state, hunt, &transition.transition) {
                        self.update_state();
                    } else {
                        log::error!("Failed to change state, resetting to idle");
                        self.app.state = HuntState::Idle;
                    }
                } else if !automatic && transition.automatic {
                    log::error!(
                        "{:?} is an automatic transition only",
                        transition.transition
                    );
                } else {
                    log::error!("Unexpected argument value for {:?}", transition.transition);
                }
            }
            None => {
                log::error!(
                    "In state {:?}, got illegal transition request {:?}",
                    self.app.state,
                    transition_req
                );
            }
        }
    }

    fn main_thread(mut self, mut shutdown_rx: oneshot::Receiver<()>) {
        let mut control = ShaooohControl::new();
        let mut vision = Vision::new();
        let mut hunt: Option<Box<dyn HuntFSM>> = None;

        while shutdown_rx.try_recv().is_err() {
            // What processing is needed
            let processing = if let Some(h) = &mut hunt {
                h.processing()
            } else {
                Vec::new()
            };
            // Frame processing
            if let Ok(results) = vision.process_next_frame(processing) {
                // Step state machines
                if let Some(h) = &mut hunt {
                    let result = h.step(&mut control, results);
                    // Automatic transition requests
                    if result.incr_encounters {
                        self.app.encounters += 1;
                        self.update_state();
                    }
                    if let Some(transition_req) = result.transition {
                        self.do_transition(transition_req, &mut hunt, true);
                    }
                }

                if let Ok(mut img_wr) = self.image.try_lock() {
                    img_wr.clear();
                    img_wr.extend(vision.read_frame());
                }
            } else {
                if !self.rx.is_closed() {
                    log::warn!("Failed to process frame");
                }
            }

            // Manual transition requests from API
            if !self.rx.is_empty() {
                if let Some(transition_req) = self.rx.blocking_recv() {
                    self.do_transition(transition_req, &mut hunt, false);
                }
            }

            if !self.button_rx.is_empty() {
                if let Some(button) = self.button_rx.blocking_recv() {
                    control.press(button);
                }
            }

            if self.rx.is_closed() {
                break;
            }

            std::thread::sleep(std::time::Duration::new(0, 50000));
        }
    }

    async fn call_webhook(mut rx: watch::Receiver<AppState>) {
        let path = "user_config.json";
        if std::fs::exists(path).unwrap_or(false) {
            let data = std::fs::read_to_string(path).expect("Couldn't read file");
            if let Ok(cfg) = serde_json::from_str::<UserConfig>(&data) {
                log::info!("Loaded user configuration");

                if let (Some(api_key), Some(user_id)) = (cfg.api_key, cfg.user_id) {
                    loop {
                        let state_copy = { Some((*rx.borrow_and_update()).clone()) };
                        if let Some(state) = state_copy {
                            //. TODO reset enc
                            let phased = state.encounters
                                - state.phases.iter().map(|x| x.encounters).max().unwrap_or(0);
                            let content = reqwest::multipart::Form::new()
                                .text(
                                    "message",
                                    format!(
                                        "State = {:?}, No. encounters = {}",
                                        &state.state, phased
                                    ),
                                )
                                .text("token", api_key.clone())
                                .text("user", user_id.clone());
                            let interesting_encounter = (phased % 64 == 0) && (phased != 0);
                            let interesting_state = (state.state != HuntState::Idle)
                                && (state.state != HuntState::Hunt);
                            if interesting_encounter || interesting_state {
                                log::info!("Calling webhook with {:?}", content);
                                let client = reqwest::Client::new();
                                match client
                                    .post("https://api.pushover.net/1/messages.json")
                                    .multipart(content)
                                    .send()
                                    .await
                                {
                                    Ok(_) => {}
                                    Err(e) => {
                                        log::error!("Failed to send webhook {:?}", e);
                                    }
                                };
                            }
                        }
                        if rx.changed().await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }

    // TODO move to separate module
    pub fn lights_thread(mut rx: watch::Receiver<AppState>) {
        const NUM_PIXELS: u32 = 7;
        let mut anim = 0;
        let mut lights = Lights::new(NUM_PIXELS, 18);
        while let Ok(_) = rx.has_changed() {
            let state_copy = { (*rx.borrow_and_update()).clone() };

            let mut data = Vec::new();

            let interesting_state =
                (state_copy.state != HuntState::Idle) && (state_copy.state != HuntState::Hunt);

            let num: u64 = NUM_PIXELS.into();
            let num_circle = num - 1;
            if interesting_state {
                let r = 0;
                let (g, b) = if state_copy.state == HuntState::FoundTarget {
                    (50, 0)
                } else {
                    (0, 50)
                };
                let w = 0;
                data.push(PixelData { r, g, b, w });
                for n in 1..num {
                    let highlight_pixel = (anim + num_circle) % num_circle == (n - 1);
                    let highlight_pixel_m1 = (anim + num_circle - 1) % num_circle == (n - 1);
                    let highlight_pixel_m2 = (anim + num_circle - 2) % num_circle == (n - 1);
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
                    let (g, b) = if state_copy.state == HuntState::FoundTarget {
                        (c, 0)
                    } else {
                        (0, c)
                    };
                    let w = 0;
                    data.push(PixelData { r, g, b, w });
                }
                anim += 1;
            } else if state_copy.state == HuntState::Idle {
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
                    let highlight_pixel =
                        (state_copy.encounters + num_circle) % num_circle == (n - 1);
                    let highlight_pixel_m1 =
                        (state_copy.encounters + num_circle - 1) % num_circle == (n - 1);
                    let highlight_pixel_m2 =
                        (state_copy.encounters + num_circle - 2) % num_circle == (n - 1);
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

            lights.draw(data);
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub async fn serve(mut self) -> Result<(), String> {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let state = self.api.take().expect("Couldn't get API state");

        let rx_clone = state.rx.clone();

        tokio::spawn(Self::call_webhook(rx_clone));

        let rx_light_clone = state.rx.clone();
        let light_thread = std::thread::spawn(|| {
            Self::lights_thread(rx_light_clone);
            log::info!("Light thread complete");
        });

        let main_thread = std::thread::spawn(|| {
            self.main_thread(shutdown_rx);
            log::info!("Main thread complete");
        });
        // run our app with hyper, listening globally on port 3000
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, Self::routes(state))
            .with_graceful_shutdown(shutdown(shutdown_tx))
            .await
            .unwrap();

        main_thread.join().expect("Error from main thread");
        light_thread.join().expect("Error from light thread");

        Ok(())
    }
}

#[derive(Clone, Serialize)]
struct ApiResponse {
    ok: bool,
    error: String,
}

#[axum::debug_handler]
async fn get_state(State(state): State<ApiState>) -> Json<ResponseAppState> {
    let state = (*state.rx.borrow()).clone();
    let transitions = state
        .state
        .possible_transitions()
        .iter()
        .filter(|x| !x.automatic)
        .cloned()
        .collect();
    Json(ResponseAppState { state, transitions })
}

#[axum::debug_handler]
async fn post_state(
    State(state): State<ApiState>,
    Json(payload): Json<RequestTransition>,
) -> Json<ApiResponse> {
    let res = state.tx.send(payload).await;
    match res {
        Ok(_) => Json(ApiResponse {
            ok: true,
            error: "".to_string(),
        }),
        Err(e) => Json(ApiResponse {
            ok: false,
            error: e.to_string(),
        }),
    }
}

#[axum::debug_handler]
async fn post_button(
    State(state): State<ApiState>,
    Json(payload): Json<Button>,
) -> Json<ApiResponse> {
    let res = state.button_tx.send(payload).await;
    match res {
        Ok(_) => Json(ApiResponse {
            ok: true,
            error: "".to_string(),
        }),
        Err(e) => Json(ApiResponse {
            ok: false,
            error: e.to_string(),
        }),
    }
}

#[axum::debug_handler]
async fn get_frame(State(state): State<ApiState>) -> impl IntoResponse {
    let headers = [
        (header::CONTENT_TYPE, "image/png"),
        (header::CACHE_CONTROL, "max-age=0, must-revalidate"),
    ];

    if let Ok(img_rd) = state.image.lock() {
        (StatusCode::OK, headers, (*img_rd).clone())
    } else {
        let vec = Vec::new();
        (StatusCode::INTERNAL_SERVER_ERROR, headers, vec.clone())
    }
}

async fn shutdown(shutdown_tx: oneshot::Sender<()>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    log::info!("Got shutdown");

    shutdown_tx
        .send(())
        .expect("Failed to send shutdown to main thread");
}
