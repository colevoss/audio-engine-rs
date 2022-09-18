use engine::Engine;
use tokio;

mod engine;
mod frame;
mod source;
mod source_reader;
mod symph;
// use tokio::sync::mpsc;

// struct IState {
//     val: u32,
// }
//
// type State = Arc<Mutex<IState>>;

#[tokio::main]
async fn main() {
    let controller = Engine::new().unwrap();

    // controller.open_source_reader("sounds/sample-1.wav".to_string());
    // controller.open_source_reader("sounds/sample-2.wav".to_string());
    // controller.open_source_reader("sounds/sample-3.wav".to_string());
    controller.open_source_reader("sounds/sample-4.wav".to_string());

    controller.play().await;

    loop {}
}
