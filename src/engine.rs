use crate::source_reader::SourceReader;
use crate::symph::Symphonia;
use crate::track::Track;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, SampleFormat, Stream, SupportedStreamConfig};
use crossbeam::channel::{bounded, select, Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;
use std::thread;
use tokio::sync::{broadcast, mpsc};

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

pub struct Engine {
    rx: Receiver<f32>,
    host: Host,
    device: Device,
    config: SupportedStreamConfig,
}

impl Engine {
    pub fn start_stream(&self) -> Result<Stream, ()> {
        let rx_clone = self.rx.clone();
        println!("Starting stream...");
        let mut config = self.config.config();
        config.buffer_size = cpal::BufferSize::Fixed(64);

        let stream_result = match self.config.sample_format() {
            SampleFormat::I16 => self.device.build_output_stream(
                // &self.config.config(),
                &config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    for s in data.iter_mut() {
                        *s = match rx_clone.recv() {
                            Ok(sample) => cpal::Sample::from::<f32>(&sample),
                            Err(_) => cpal::Sample::from::<f32>(&0f32),
                        }
                    }
                },
                err_fn,
            ),
            SampleFormat::U16 => self.device.build_output_stream(
                // &self.config.config(),
                &config,
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    for s in data.iter_mut() {
                        *s = match rx_clone.recv() {
                            Ok(sample) => cpal::Sample::from::<f32>(&sample),
                            Err(_) => cpal::Sample::from::<f32>(&0f32),
                        }
                    }
                },
                err_fn,
            ),
            SampleFormat::F32 => self.device.build_output_stream(
                // &self.config.config(),
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    for s in data.iter_mut() {
                        *s = match rx_clone.recv() {
                            Ok(sample) => cpal::Sample::from::<f32>(&sample),
                            Err(_) => cpal::Sample::from::<f32>(&0f32),
                        }
                    }
                },
                err_fn,
            ),
        };

        let stream = stream_result.unwrap();

        Ok(stream)
    }

    // pub fn send(&self, sample: f32) {
    // }
}

pub struct EngineController {
    sources: Arc<Mutex<Sources>>,
    tracks: Vec<Arc<Track>>,
    engine: Arc<Mutex<Engine>>,
    command_tx: Sender<EngineCommand>,
    command_rx: Receiver<EngineCommand>,
    // command_tx: broadcast::Sender<EngineCommand>,
    // command_rx: broadcast::Receiver<EngineCommand>,
    pub prod: Sender<f32>,
    config: SupportedStreamConfig,
    engine_state: Arc<RwLock<EngineStateMachine>>,
}

impl EngineController {
    pub fn new() -> Result<EngineController, ()> {
        // TODO: Make sure this buffer is good.
        let (tx, rx) = bounded::<f32>(1024);
        let (command_tx, command_rx) = bounded::<EngineCommand>(1);
        // let (mut command_tx, command_rx) = broadcast::channel::<EngineCommand>(1);

        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();

        println!("Default Output: {:?}", config);

        let engine = Engine {
            rx,
            host,
            config: config.clone(),
            device,
        };

        let controller = EngineController {
            engine_state: Arc::new(RwLock::new(EngineStateMachine::new())),
            prod: tx,
            command_tx,
            command_rx,
            sources: Arc::new(Mutex::new(Sources::new())),
            engine: Arc::new(Mutex::new(engine)),
            tracks: Default::default(),
            config: config.clone(),
        };

        Ok(controller)
    }

    pub fn engine(&self) -> Arc<Mutex<Engine>> {
        self.engine.clone()
    }

    pub fn add(&self, decoder: Symphonia) {
        let reader = SourceReader::new(decoder, self.config.config());
        self.sources.lock().add(reader);
    }

    pub fn open_source_reader(&self, path: String) {
        let source = Symphonia::new(path).unwrap();
        self.add(source);
    }

    // pub async fn play(&self) {
    pub fn play(&self) {
        let sources = self.sources.clone();
        let sender = self.prod.clone();
        let engine = self.engine.clone();
        let command_rx = self.command_rx.clone();
        let state_machine = self.engine_state.clone();

        thread::spawn(move || {
            let engine_lock = engine.lock();
            let stream = engine_lock.start_stream().unwrap();

            loop {}
        });

        // Starting the stream can probably be moved to the other spawned process below
        // tokio::spawn(async move {
        //     let engine_lock = engine.lock();
        //     let stream = engine_lock.start_stream().unwrap();
        //
        //     loop {}
        // });

        // tokio::spawn(async move {
        // thread::spawn(move || {
        //     let mut sources_lock = sources.lock();
        //     while let Some(sample) = sources_lock.next() {
        //         match sender.send(sample) {
        //             Err(err) => eprintln!("Send Err: {}", err),
        //             _ => (),
        //         }
        //     }
        // });

        self.engine_state.write().play();
    }

    pub fn play_tracks(&self) {}
}

struct Sources {
    source_readers: Vec<SourceReader>,
}

impl Sources {
    pub fn new() -> Sources {
        Sources {
            source_readers: Default::default(),
        }
    }

    pub fn add(&mut self, reader: SourceReader) {
        self.source_readers.push(reader);
    }
}

impl Iterator for Sources {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        let mut samp = 0f32;
        let mut full_decoders = self.source_readers.len();

        for source in self.source_readers.iter_mut() {
            match source.next() {
                Some(sample) => {
                    samp = (samp + sample).clamp(-1f32, 1f32);
                }
                None => {
                    full_decoders = full_decoders - 1;

                    if full_decoders == 0 {
                        return None;
                    }
                }
            };
        }

        Some(samp)
    }
}

#[derive(Debug, Clone)]
enum EngineState {
    Idle,
    Playing,
    Paused,
}

#[derive(Debug)]
enum EngineCommand {
    Play,
    Pause,
}

struct EngineStateMachine {
    state: EngineState,
}

impl EngineStateMachine {
    pub fn new() -> Self {
        EngineStateMachine {
            state: EngineState::Idle,
        }
    }

    pub fn next(&mut self, comm: EngineCommand) {
        self.state = match (self.state.clone(), comm) {
            (EngineState::Idle, EngineCommand::Play) => EngineState::Playing,
            (EngineState::Playing, EngineCommand::Pause) => EngineState::Paused,
            (EngineState::Paused, EngineCommand::Play) => EngineState::Playing,
            (s, _) => s,
        };
    }

    pub fn play(&mut self) {
        self.next(EngineCommand::Play);
    }
}
