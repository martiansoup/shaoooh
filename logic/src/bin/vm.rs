use std::time::Duration;
use std::time::SystemTime;

use shaoooh::app::{Game, Method};

use env_logger::Env;

use clap::Parser;

use shaoooh::vm::FsmParser;

use shaoooh::vision::ProcessingResult;

/// Shaoooh - Simulate a hunt state machine
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// FSM description file
    fsm: String,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    log::info!("Starting Shaoooh VM : Hunt Simulator");

    let args = Args::parse();

    log::info!("Loading fsm file: {}", args.fsm);

    let fsm_file = std::fs::read(args.fsm).expect("Failed to read FSM file");

    let parser = FsmParser::new(fsm_file);

    let parsed_fsm = parser.parse();

    log::trace!("Parsed: {:#?}", parsed_fsm);

    if let Some(parsed) = parsed_fsm {
        // TODO Game/Method
        let target = 1;
        let fsm = parsed.build(target, Game::FireRedLeafGreen, Method::SoftResetGift, false);

        match fsm {
            Ok(mut f) => {
                log::trace!("Built: {:#?}", f);

                log::info!("Start state: {}", f.current_name());

                let time = SystemTime::now();
                while time.elapsed().unwrap() < Duration::from_secs(10) {
                    let inputs = f.inputs();
                    let mut results = vec![];
                    if inputs.len() > 0 {
                        for i in inputs {
                            println!("Value for input '{:?}': (0 - not met, 1 - met, 2 - shiny, 3 - non-target)", i);
                            let mut buffer = String::new();
                            std::io::stdin().read_line(&mut buffer).expect("Failed to read line");
                            let proc = i.clone();
                            let (met, shiny, species) = match buffer.trim() {
                                "0" => (false, false, 0),
                                "1" => (true, false, 0),
                                "2" => (true, true, target),
                                "3" => (true, true, 0),
                                _ => (false, false, 0)
                            };

                            results.push(ProcessingResult {
                                process: proc,
                                met,
                                shiny,
                                species: 0
                            })
                        }
                    }
                    let outputs = f.outputs();
                    for o in outputs {
                        log::info!("  Output = {}", o);
                    }
                    f.process(results);
                    std::thread::sleep(Duration::from_millis(5));
                }

            }
            Err(e) => {
                log::error!("Couldn't build FSM: {:?}", e);
            }
        }
    }

}

//   pub fn step(
//         &mut self,
//         control: &mut Box<dyn BotControl>,
//         results: Vec<ProcessingResult>,
//     ) -> HuntResult {
//         let outputs = self.fsm.outputs();
//         if !outputs.is_empty() {
//             if outputs.windows(2).all(|v| v[0].delay == v[1].delay) {
//                 // All delays match, press buttons togther
//                 let buttons: Vec<&Button> = outputs.iter().map(|v| &v.button).collect();
//                 control.presses_delay(buttons.as_slice(), &outputs[0].delay);
//             } else {
//                 // Buttons have different delays, press in sequence
//                 for out in outputs {
//                     control.press_delay(&out.button, &out.delay);
//                 }
//             }
//         }

//         self.step_no_output(results)
//     }

//     pub fn cleanup(&mut self) {}

//     pub fn current_name(&self) -> String {
//         self.fsm.current_name()
//     }

//     pub fn debug_name(&self) -> String {
//         self.fsm.debug_name()
//     }

//     // Only public for testing
//     pub fn step_no_output(&mut self, results: Vec<ProcessingResult>) -> HuntResult {
//         if let Some(output) = self.fsm.process(results) {
//             output
//         } else {
//             HuntResult {
//                 transition: None,
//                 incr_encounters: false,
//             }
//         }
//     }