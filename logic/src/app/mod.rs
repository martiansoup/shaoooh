use std::{
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{Arc, Mutex, atomic::AtomicBool},
    thread::JoinHandle,
    time::Duration,
};

use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use opencv::core::Mat;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, watch};
use tokio_util::sync::CancellationToken;
use tower_http::services::ServeDir;
use uuid::{Uuid, uuid};
pub(crate) mod error;
pub(crate) mod main;
pub(crate) mod states;
use crate::{
    control::{
        BishaanControl, BishaanControlSocket, BotControl, Button, Delay, NopControl, ShaooohControl,
    },
    displays::{DisplayWrapper, GfxDisplay, Webhook},
    hunt::{HuntBuild, HuntFSM},
    vision::{BishaanVision, BishaanVisionSocket, BotVision, NopVision, Vision},
};
pub use error::*;
pub use states::*;
use tokio::signal;

pub use main::*;

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
use crate::displays::LightsDisplay;

// Response to any requests for the current state, also includes possible transitions
#[derive(Clone, Serialize)]
struct ResponseAppState {
    state: AppState,
    transitions: Vec<StateTransition>,
}

#[derive(Clone, Serialize)]
struct ResponseMode {
    extended: bool,
    info: String,
    description: String,
    emoji: String,
}

#[derive(Clone)]
struct ApiState {
    rx: watch::Receiver<AppState>,
    tx: mpsc::Sender<RequestTransition>,
    tx_conn: watch::Sender<bool>,
    button_tx: mpsc::Sender<(Button, Delay)>,
    image: Arc<Mutex<Vec<u8>>>,
    image2: Arc<Mutex<Vec<u8>>>,
    mode: ResponseMode,
}

pub struct Shaoooh {
    api: Option<ApiState>,
    app: AppState,
    tx: watch::Sender<AppState>,
    rx: mpsc::Receiver<RequestTransition>,
    rx_conn: watch::Receiver<bool>,
    button_rx: mpsc::Receiver<(Button, Delay)>,
    error_tx: Arc<broadcast::Sender<ShaooohError>>,
    image: Arc<Mutex<Vec<u8>>>,
    image2: Arc<Mutex<Vec<u8>>>,
    config: Config,
    atomic: Arc<AtomicBool>,
}

// Struct to load/save from disc
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HuntInformation {
    pub name: String,
    pub species: u32,
    pub game: Game,
    pub method: Method,
    pub encounters: u64,
    pub phases: Vec<Phase>,
    pub complete: bool,
    pub date: Option<DateTime<Utc>>,
}

impl Shaoooh {
    pub fn new(config: Config) -> Self {
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
        let (conn_tx, conn_rx) = watch::channel(false);
        let image_mutex = Arc::new(Mutex::new(Vec::new()));
        let image_mutex2 = Arc::new(Mutex::new(Vec::new()));
        let atomic = Arc::new(AtomicBool::new(true));
        // RX will subscribe later from TX reference
        let (error_tx_chnl, _error_rx) = broadcast::channel(32);
        let error_tx = Arc::new(error_tx_chnl);

        // Support 3ds features
        let mode_tup = match config {
            Config::Bishaan(..) => (true, config.info(), config.description(), config.emoji()),
            _ => (false, config.info(), config.description(), config.emoji()),
        };
        let mode = ResponseMode {
            extended: mode_tup.0,
            info: mode_tup.1,
            description: mode_tup.2,
            emoji: mode_tup.3,
        };

        let api = ApiState {
            rx: state_rx,
            tx: transition_tx,
            tx_conn: conn_tx,
            button_tx,
            image: image_mutex.clone(),
            image2: image_mutex2.clone(),
            mode,
        };
        Self {
            api: Some(api),
            app,
            tx: state_tx,
            rx: transition_rx,
            rx_conn: conn_rx,
            button_rx,
            error_tx,
            image: image_mutex,
            image2: image_mutex2,
            config,
            atomic,
        }
    }

    fn routes(state: ApiState) -> Router {
        let static_dir = ServeDir::new("./static");

        Router::new()
            .route("/", get(get_index))
            .nest_service("/static", static_dir)
            .route("/api/state", get(get_state).post(post_state))
            .route("/api/button", post(post_button))
            .route("/api/frame", get(get_frame))
            .route("/api/frame2", get(get_frame2))
            .route("/api/mode", get(get_mode))
            .with_state(state)
    }

    fn filename_from_name(name: &str) -> String {
        format!("hunts/hunt_{}.json", name)
    }

    pub fn get_all_hunt_files() -> Vec<PathBuf> {
        std::fs::read_dir("hunts/")
            .expect("Failed to read hunts")
            .filter_map(|p| {
                if let Ok(d) = p
                    && let Ok(f) = d.file_type()
                    && f.is_file()
                    && d.path().extension().is_some_and(|x| x == "json")
                {
                    return Some(d.path());
                }
                None
            })
            .collect()
    }

    pub fn get_all_hunts() -> Vec<HuntInformation> {
        let files = Self::get_all_hunt_files();
        let mut res = Vec::new();

        for f in files {
            let data = std::fs::read_to_string(f).expect("Couldn't read file");
            let hunt: HuntInformation = serde_json::from_str(&data).expect("Failed to parse json");
            res.push(hunt);
        }

        res
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
                date: if self.app.state == HuntState::FoundTarget {
                    Some(Utc::now())
                } else {
                    None
                },
            };
            let file = std::fs::File::create(Self::filename_from_name(&name)).unwrap();
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, &state).expect("Failed to serialise state");
            writer.flush().expect("Failed to flush to file");
        }
        self.tx
            .send(self.app.clone())
            .expect("Couldn't update state");
    }

    fn transition_logic(
        &mut self,
        from: HuntState,
        hunt: &mut Option<HuntFSM>,
        transition: &Transition,
    ) -> bool {
        if self.app.state == HuntState::Hunt && from != HuntState::Hunt {
            // Build hunt object
            let target = self.app.arg.as_ref().unwrap().species;
            let game = self.app.arg.as_ref().unwrap().game.clone();
            let method = self.app.arg.as_ref().unwrap().method.clone();
            let new_hunt = HuntBuild::build(target, game, method, self.atomic.clone());
            match new_hunt {
                Some(h) => *hunt = Some(h),
                None => return false,
            };
        }
        let phased = (self.app.state != HuntState::FoundNonTarget
            && from == HuntState::FoundNonTarget
            && *transition != Transition::FalseDetect)
            || (self.app.state != HuntState::FoundTarget
                && from == HuntState::FoundTarget
                && *transition != Transition::Fail
                && *transition != Transition::FalseDetect);
        if phased {
            // If phased transition - species was unknown as came via FoundTarget, has to be fixed manually
            log::warn!("Unknown species for phase, needs manual update");
            let phase = Phase {
                caught: *transition == Transition::Caught,
                species: if *transition == Transition::Phased {
                    0
                } else {
                    self.app.last_phase
                },
                encounters: self.app.encounters,
                date: Utc::now(),
            };
            self.app.phases.push(phase);
            // Reset the encounters after a phase
            self.app.encounters = 0;
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
        hunt: &mut Option<HuntFSM>,
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

    fn main_thread(
        mut self,
        top_frame_rx: watch::Receiver<Mat>,
        bottom_frame_rx: watch::Receiver<Mat>,
        button_tx: mpsc::Sender<(Vec<Button>, Delay)>,
        shutdown_token: CancellationToken,
        raw_frame_mutex: Arc<Mutex<Mat>>,
    ) {
        let (mut control, mut vision): (Box<dyn BotControl>, Box<dyn BotVision>) = match self.config
        {
            Config::Shaoooh(ref cfg) => (
                Box::new(ShaooohControl::new(cfg.control())),
                Box::new(Vision::new(cfg.video(), raw_frame_mutex)),
            ),
            Config::Bishaan(_) => (
                Box::new(BishaanControl::new(button_tx)),
                Box::new(BishaanVision::new(top_frame_rx, bottom_frame_rx)),
            ),
            Config::Ditto => (Box::new(NopControl::new()), Box::new(NopVision::new())),
        };
        let mut hunt: Option<HuntFSM> = None;

        while !shutdown_token.is_cancelled() {
            // What processing is needed
            let processing = if let Some(h) = &mut hunt {
                h.processing()
            } else {
                &Vec::new()
            };
            // Frame processing
            if let Some(results) = vision.process_next_frame(processing) {
                // Step state machines
                if let Some(h) = &mut hunt {
                    let result = h.step(&mut control, results);
                    h.display();
                    // Automatic transition requests
                    if result.incr_encounters {
                        self.app.encounters += 1;
                        log::info!("Current encounters: {}", self.app.encounters);
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
                if let Ok(mut img_wr) = self.image2.try_lock() {
                    img_wr.clear();
                    img_wr.extend(vision.read_frame2());
                }
            } else if !self.rx.is_closed() {
                log::warn!("Failed to process frame");
            }

            // Manual transition requests from API
            if !self.rx.is_empty()
                && let Some(transition_req) = self.rx.blocking_recv()
            {
                self.do_transition(transition_req, &mut hunt, false);
            }

            if !self.button_rx.is_empty()
                && let Some((button, delay)) = self.button_rx.blocking_recv()
            {
                control.press_delay(&button, &delay);
            }

            if self.rx.is_closed() {
                break;
            }

            std::thread::sleep(std::time::Duration::new(0, 500000));
        }
    }

    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    fn add_lights_display(displays: &mut Vec<DisplayWrapper>) {
        displays.push(DisplayWrapper::new(
            "Neopixel display".to_string(),
            Box::new(|| Box::new(LightsDisplay::default())),
        ));
    }

    #[cfg(not(all(target_arch = "aarch64", target_os = "linux")))]
    fn add_lights_display(_: &mut Vec<DisplayWrapper>) {}

    pub fn get_service(&self) -> async_zeroconf::Service {
        let mut txt = async_zeroconf::TxtRecord::new();
        txt.add("info".to_string(), self.config.info().clone());
        txt.add("descr".to_string(), self.config.description().clone());
        txt.add("emoji".to_string(), self.config.emoji().clone());

        let namespace = uuid!("3eb75a82-cc0a-4e21-8db1-4936e5ee03a8");
        let uuid = Uuid::new_v5(&namespace, self.config.short().as_bytes());
        let uuid_str = format!("{}", uuid);
        txt.add("uuid".to_string(), uuid_str);
        async_zeroconf::Service::new_with_txt(&self.config.name(), "_shaoooh._tcp", 3000, txt)
    }

    pub fn serve(mut self, skip_conn: bool) -> std::io::Result<()> {
        let service = self.get_service();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = runtime.enter();

        let (service_ref, task, service_ok) =
            service.publish_task().expect("Failed to create service");

        runtime.spawn(task);

        runtime
            .block_on(service_ok)
            .expect("Failed to publish service");

        log::info!(
            "Selected configuration: {} {}",
            self.config.emoji(),
            self.config.info()
        );
        log::info!("  {}", self.config.description());
        log::info!(
            "OpenCV version: {}.{}.{}",
            opencv::core::CV_VERSION_MAJOR,
            opencv::core::CV_VERSION_MINOR,
            opencv::core::CV_VERSION_REVISION
        );
        let shutdown_token = CancellationToken::new();

        let state = self.api.take().expect("Couldn't get API state");
        let rx_clone_hook = state.rx.clone();
        let rx_clone_disp = state.rx.clone();
        let runtime_hndl = runtime.handle().clone();
        let error_rx_shutdown = self.error_tx.subscribe();

        // Have to start web server early to catch connection requests
        // from the 3DS
        let shutdown_token_server = shutdown_token.clone();
        let handle = std::thread::spawn(move || {
            runtime_hndl.block_on(async {
                // run our app with hyper, listening globally on port 3000
                let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
                axum::serve(listener, Self::routes(state))
                    .with_graceful_shutdown(shutdown(shutdown_token_server, error_rx_shutdown))
                    .await
                    .expect("Error from web server");
            })
        });

        // If in Bishaan(3DS) configuration, want to wait until the 3DS has performed a connection
        // test, and then allow some time to start InputRedirection and Streaming
        if !skip_conn {
            if let Config::Bishaan(_) = self.config {
                let shutdown_token_conn = shutdown_token.clone();
                runtime.block_on(async {
                    tokio::select! {
                        _ = shutdown_token_conn.cancelled() => {
                            log::info!("Got shutdown during connection test");
                        }
                        _ = async {
                            log::info!("Waiting for connection test to be performed");
                            self.rx_conn.changed().await.unwrap();
                            log::info!(
                                "Seen connection test, waiting for a minute to enable InputRedirection and NTR"
                            );
                            // TODO report time in 10 second intervals to count down
                            for x in 0..5 {
                                log::info!("{} seconds remaining...", 60 - (x * 10));
                                tokio::time::sleep(Duration::from_secs(10)).await;
                            }
                            for x in 0..10 {
                                log::info!("{} seconds remaining...", 10 - x);
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        } => {
                            log::info!("Resuming startup...");
                        }
                    };
                });
            }
        }

        let (t_frame_tx, t_frame_rx) = watch::channel(Mat::default());
        let (b_frame_tx, b_frame_rx) = watch::channel(Mat::default());
        let (button_tx, button_rx) = mpsc::channel(16);
        let error_rx_webhook = self.error_tx.subscribe();

        runtime.spawn(Webhook::call(
            rx_clone_hook,
            error_rx_webhook,
            self.config.name(),
        ));

        let mut displays: Vec<DisplayWrapper> = Vec::new();
        let mut handles: Vec<(String, JoinHandle<()>)> = Vec::new();
        let atomic_clone = self.atomic.clone();
        let error_tx_clone = self.error_tx.clone();

        log::info!("Adding state listeners and communication threads");
        match self.config {
            Config::Shaoooh(_) => {
                log::info!("- Neopixels");
                Self::add_lights_display(&mut displays);
                log::info!("- Counter screen");
                displays.push(DisplayWrapper::new(
                    "Gfx Screen".to_string(),
                    Box::new(|| Box::new(GfxDisplay::default())),
                ));
            }
            Config::Bishaan(ip) => {
                runtime.spawn(async move {
                    log::info!("- Frame stream Rx thread");
                    let vision = BishaanVisionSocket::new(
                        ip,
                        t_frame_tx,
                        b_frame_tx,
                        atomic_clone,
                        error_tx_clone,
                    )
                    .await
                    .expect("Error creating vision thread");
                    let vision_handle = tokio::spawn(vision.task());
                    log::info!("- Control Tx thread");
                    let control = BishaanControlSocket::new(ip, button_rx)
                        .await
                        .expect("Error creating control thread");
                    let control_handle = tokio::spawn(control.task());

                    tokio::select! {
                        r = vision_handle => {
                            if let Err(e) = r {
                                log::error!("Error from vision thread: {:?}", e);
                            }
                        }
                        r = control_handle => {
                            if let Err(e) = r {
                                log::error!("Error from control thread: {:?}", e);
                            }
                        }
                    }
                });
            }
            Config::Ditto => {
                log::info!("- No listeners");
            }
        };

        let raw_frame_mutex = Arc::new(Mutex::new(Mat::default()));
        // TODO allow enabling display
        //let mutex_copy = raw_frame_mutex.clone();
        //displays.push(DisplayWrapper::new(
        //    "Screen display".to_string(),
        //    Box::new(move || Box::new(ScreenDisplay::new(mutex_copy))),
        //));

        for mut display in displays {
            let rx_clone = rx_clone_disp.clone();
            let name = display.name();
            log::info!("Creating thread for display: '{}'", name);
            let handle = std::thread::spawn(move || {
                display.thread(rx_clone);
            });
            handles.push((name, handle));
        }

        self.main_thread(
            t_frame_rx,
            b_frame_rx,
            button_tx,
            shutdown_token.clone(),
            raw_frame_mutex,
        );
        log::info!("Main thread complete");

        handle.join().expect("Error from server thread");
        // TODO should wait for all threads including 3ds communication threads

        for handle in handles {
            handle
                .1
                .join()
                .unwrap_or_else(|_| panic!("Error from thread: {}", handle.0))
        }

        Ok(())
    }
}

#[derive(Clone, Serialize)]
struct ApiResponse {
    ok: bool,
    error: String,
}

#[axum::debug_handler]
async fn get_index(
    State(state): State<ApiState>,
    req: axum::extract::Request<axum::body::Body>,
) -> impl IntoResponse {
    let headers = [
        (header::CONTENT_TYPE, "text/html"),
        (header::CONNECTION, "keep-alive"),
        (
            header::HeaderName::from_static("x-organization"),
            "Nintendo",
        ),
    ];

    if req.uri() == "http://conntest.nintendowifi.net/" {
        log::info!("Got connection test request");
        let _ = state.tx_conn.send(true);
    }
    std::fs::read("index.html")
        .map(|b| (StatusCode::OK, headers, b))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
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
    let delay = Delay::Tenth;
    let res = state.button_tx.send((payload, delay)).await;
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
        (
            header::CACHE_CONTROL,
            "no-cache, must-revalidate, max-age=0, no-store",
        ),
    ];

    if let Ok(img_rd) = state.image.lock() {
        (StatusCode::OK, headers, (*img_rd).clone())
    } else {
        let vec = Vec::new();
        (StatusCode::INTERNAL_SERVER_ERROR, headers, vec.clone())
    }
}

#[axum::debug_handler]
async fn get_frame2(State(state): State<ApiState>) -> impl IntoResponse {
    let headers = [
        (header::CONTENT_TYPE, "image/png"),
        (
            header::CACHE_CONTROL,
            "no-cache, must-revalidate, max-age=0, no-store",
        ),
    ];

    if let Ok(img_rd) = state.image2.lock() {
        (StatusCode::OK, headers, (*img_rd).clone())
    } else {
        let vec = Vec::new();
        (StatusCode::INTERNAL_SERVER_ERROR, headers, vec.clone())
    }
}

#[axum::debug_handler]
async fn get_mode(State(state): State<ApiState>) -> Json<ResponseMode> {
    Json(state.mode)
}

async fn shutdown(
    shutdown_token: CancellationToken,
    mut error_rx: broadcast::Receiver<ShaooohError>,
) {
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
        err = error_rx.recv() => {
            match err {
                Ok(e) => log::error!("Got Error [{}], shutting down", e),
                Err(_) => log::error!("Error channel dropped")
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    log::info!("Got shutdown");

    shutdown_token.cancel();
}
