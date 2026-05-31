use std::sync::{Arc, atomic::AtomicBool};

use simple_logger::SimpleLogger;

use shaoooh::app::{Game, Method};
use shaoooh::hunt::HuntBuild;

use clap::Parser;

/// Shaoooh - Generate FSM of a hunt
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// Game
    #[arg(value_enum)]
    game: Game,
    #[arg(value_enum)]
    /// Method
    method: Method,
    /// Target
    target: u32,
}

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::Level::Debug.to_level_filter())
        .with_utc_timestamps()
        .init()
        .unwrap();

    log::info!("Starting Shaoooh : FSM Generation");

    let args = Args::parse();

    let atom = Arc::new(AtomicBool::default());

    if let Some(mut fsm) = HuntBuild::build(args.target, args.game, args.method, atom) {
        log::info!("Created state machine");
        fsm.graph_file("graph")
            .expect("Failed to create graph file");
    }
}
