use std::thread;
use std::time::Duration;

use chrono::Datelike;
use shaoooh::context::PkContext;

use simple_logger::SimpleLogger;

use shaoooh::app::{Game, Method, Shaoooh};
use shaoooh::hunt::HuntBuild;

use clap::Parser;

use shaoooh::vm::FsmParser;

/// Shaoooh - Simulate a hunt state machine
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// FSM description file
    fsm: String,
}

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::Level::Debug.to_level_filter())
        .with_utc_timestamps()
        .init()
        .unwrap();

    log::info!("Starting Shaoooh VM : Hunt Simulator");

    let args = Args::parse();

    log::info!("Loading fsm file: {}", args.fsm);

    let fsm_file = std::fs::read(args.fsm).expect("Failed to read FSM file");

    let parser = FsmParser::new(fsm_file);

    let fsm = parser.parse();

    log::info!("Parsed: {:#?}", fsm);

}