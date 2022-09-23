pub use crate::builder::*;
use engine::{Engine, EngineController};
use tokio;

pub mod builder;
mod channer;
mod engine;
mod frame;
mod sample_rate;
mod source;
mod source_reader;
mod symph;
mod track;

// use tokio::sync::mpsc;

// struct IState {
//     val: u32,
// }
//
// type State = Arc<Mutex<IState>>;

#[tokio::main]
async fn main() {
    let controller = EngineController::new().unwrap();

    // controller.open_source_reader("sounds/sample-1.wav".to_string());
    // controller.open_source_reader("sounds/sample-2.wav".to_string());
    // controller.open_source_reader("sounds/sample-3.wav".to_string());
    // controller.open_source_reader("sounds/sample-3.wav".to_string());
    controller.open_source_reader("sounds/sample-3.wav".to_string());
    // controller.open_source_reader("sounds/sample-3.wav".to_string());
    // controller.open_source_reader("sounds/sample-3.wav".to_string());
    // controller.open_source_reader("sounds/sample-4.wav".to_string());
    // controller.open_source_reader("sounds/sample-5.wav".to_string());
    // controller.open_source_reader("sounds/silence.wav".to_string());

    controller.play().await;

    loop {}
}
