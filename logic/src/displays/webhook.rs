use serde::Deserialize;
use tokio::sync::{broadcast, watch};

use crate::{
    app::{AppState, HuntState, ShaooohError},
    context::PkContext,
};

#[derive(Deserialize)]
struct UserConfig {
    api_key: Option<String>,
    user_id: Option<String>,
}

pub struct Webhook {}

impl Webhook {
    pub async fn error(err: ShaooohError, name: &String, api_key: String, user_id: String) {
        let title = format!("{} Error", name);
        let content = reqwest::multipart::Form::new()
            .text("message", format!("ERROR = {}", err))
            .text("token", api_key.clone())
            .text("user", user_id.clone())
            .text("title", title)
            .text("priority", "1");
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

    pub async fn status(
        state_copy: Option<AppState>,
        name: &String,
        api_key: String,
        user_id: String,
    ) {
        if let Some(state) = state_copy
            && let Some(arg) = state.arg
        {
            let interesting_state =
                (state.state != HuntState::Idle) && (state.state != HuntState::Hunt);
            // TODO last found result? rather than only phase
            let species = if state.state == HuntState::FoundNonTarget {
                state.last_phase
            } else {
                arg.species
            };
            let path_png = PkContext::get().sprite_path(&arg.game, species, interesting_state);
            let path = if std::fs::exists(&path_png).unwrap() {
                path_png
            } else {
                panic!("Couldn't get reference image {}", path_png)
            };
            let phased = state.encounters;
            let interesting_encounter = (phased % 64 == 0) && (phased != 0);
            let title = if interesting_state {
                format!("{} Alert", name)
            } else {
                format!("{} Status", name)
            };
            let content = reqwest::multipart::Form::new()
                .text(
                    "message",
                    format!("State = {:?}, No. encounters = {}", &state.state, phased),
                )
                .text("token", api_key.clone())
                .text("user", user_id.clone())
                .text("title", title)
                .text("attachment_type", "image/png")
                .file("attachment", path)
                .await
                .unwrap();
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
    }

    pub async fn call(
        mut rx: watch::Receiver<AppState>,
        mut error_rx: broadcast::Receiver<ShaooohError>,
        name: String,
    ) {
        let path = "user_config.json";
        if std::fs::exists(path).unwrap_or(false) {
            let data = std::fs::read_to_string(path).expect("Couldn't read file");
            if let Ok(cfg) = serde_json::from_str::<UserConfig>(&data) {
                log::info!("Loaded user configuration");

                if let (Some(api_key), Some(user_id)) = (cfg.api_key, cfg.user_id) {
                    loop {
                        let end = tokio::select! {
                            err = error_rx.recv() => {
                                match err {
                                    Ok(err) => {
                                       Self::error(err, &name, api_key.clone(), user_id.clone()).await;
                                       false
                                    }
                                    Err(_) => true
                                }
                            }
                            rx_val = rx.changed() => {
                                match rx_val {
                                    Ok(_) => {
                                        let state_copy = { Some((*rx.borrow_and_update()).clone()) };
                                        Self::status(state_copy, &name, api_key.clone(), user_id.clone()).await;
                                        false
                                    },
                                    Err(_) => true
                                }
                            }
                        };
                        if end {
                            break;
                        }
                    }
                }
            }
        }
    }
}
