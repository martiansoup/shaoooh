// use std::thread;
// use std::time::Duration;

// use shaoooh::vision::{Processing, ProcessingResult};
use simple_logger::SimpleLogger;

use shaoooh::app::{Game, Method};
use shaoooh::hunt::HuntBuild;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::Level::Debug.to_level_filter())
        .with_utc_timestamps()
        .init()
        .unwrap();

    log::info!("Starting Shaoooh Test");

    if let Some(fsm) = HuntBuild::build(19, Game::FireRedLeafGreen, Method::RandomEncounter) {
        log::info!("Created state machine");
        log::info!("{:#?}", fsm);

        // log::info!(
        //     "Current state {} - {}",
        //     fsm.current_name(),
        //     fsm.debug_name()
        // );
        // let results = vec![ProcessingResult {
        //     process: Processing::FRLG_START_ENCOUNTER,
        //     met: true,
        //     species: 0,
        //     shiny: false,
        // }];
        // fsm.step_no_output(results);
        // thread::sleep(Duration::from_secs(1));
        // log::info!(
        //     "Current state {} - {}",
        //     fsm.current_name(),
        //     fsm.debug_name()
        // );
        // let results = vec![ProcessingResult {
        //     process: Processing::FRLG_START_ENCOUNTER,
        //     met: true,
        //     species: 0,
        //     shiny: false,
        // }];
        // fsm.step_no_output(results);
        // for _ in 0..100 {
        //     log::info!(
        //         "Loop Current state {} - {}",
        //         fsm.current_name(),
        //         fsm.debug_name()
        //     );
        //     fsm.step_no_output(Vec::new());
        //     thread::sleep(Duration::from_millis(50));
        // }
    } else {
        log::error!("Failed to build state machine");
    }
}
