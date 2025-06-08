use axum::{Json, Router, extract::State, routing::get, routing::post};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot, watch};
use tower_http::services::{ServeDir, ServeFile};
mod states;
use states::*;
use tokio::signal;
use crate::control::{Button, ShaooohControl};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct RequestTransition {
    transition: Transition,
    arg: Option<TransitionArg>,
}

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
    button_tx: mpsc::Sender<Button>
}

pub struct Shaoooh {
    api: Option<ApiState>,
    app: AppState,
    tx: watch::Sender<AppState>,
    rx: mpsc::Receiver<RequestTransition>,
    button_rx: mpsc::Receiver<Button>
}

impl Shaoooh {
    pub fn new() -> Self {
        let app = AppState {
            state: HuntState::Idle,
            arg: None,
            encounters: 0,
        };
        let (state_tx, state_rx) = watch::channel(app.clone());
        let (transition_tx, transition_rx) = mpsc::channel(1);
        let (button_tx, button_rx) = mpsc::channel(8);

        let api = ApiState {
            rx: state_rx,
            tx: transition_tx,
            button_tx: button_tx
        };
        Self {
            api: Some(api),
            app,
            tx: state_tx,
            rx: transition_rx,
            button_rx: button_rx
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
            .with_state(state)
    }

    fn main_thread(mut self, mut shutdown_rx: oneshot::Receiver<()>) -> () {
        let mut control = ShaooohControl::new();

        while let Err(_) = shutdown_rx.try_recv() {
            // Manual transition requests from
            if !self.rx.is_empty() {
                if let Some(transition_req) = self.rx.blocking_recv() {
                    let possible_transitions = self.app.state.possible_transitions();
                    let try_transition = possible_transitions
                        .iter()
                        .find(|x| x.transition == transition_req.transition);
                    match try_transition {
                        Some(transition) => {
                            log::info!("Got transition {:?}", transition);
                            let arg = transition_req.arg;
                            if arg.is_some() == transition.needs_arg && !transition.automatic {
                                self.app.state = transition.next_state.clone();
                                if transition.needs_arg {
                                    self.app.arg = Some(arg.unwrap());
                                    self.app.encounters = 0; // TODO read previous encounters if from file
                                    log::info!("Got argument: {:?}", self.app.arg);
                                }
                                self.tx
                                    .send(self.app.clone())
                                    .expect("Couldn't update state");
                            } else {
                                if transition.automatic {
                                    log::error!(
                                        "{:?} is an automatic transition only",
                                        transition.transition
                                    );
                                } else {
                                    log::error!(
                                        "Unexpected argument value for {:?}",
                                        transition.transition
                                    );
                                }
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

    pub async fn serve(mut self) -> Result<(), String> {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let state = self.api.take().expect("Couldn't get API state");

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

    shutdown_tx
        .send(())
        .expect("Failed to send shutdown to main thread");
}
