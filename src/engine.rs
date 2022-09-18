use crate::source_reader::SourceReader;
use crate::symph::Symphonia;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, SampleFormat, Stream, SupportedStreamConfig};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use parking_lot::Mutex;
use std::sync::Arc;

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

pub struct Engine {
    command_rx: Receiver<EngineCommand>,
    // controller: Arc<EngineController>,
    rx: Receiver<f32>,
    host: Host,
    device: Device,
    config: SupportedStreamConfig,
}

enum EngineCommand {
    Play,
}

impl Engine {
    // pub fn new() -> Result<(Engine, Arc<EngineController>), ()> {
    pub fn new() -> Result<EngineController, ()> {
        let (tx, rx) = bounded::<f32>(1024);
        let (command_tx, command_rx) = unbounded::<EngineCommand>();

        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();

        println!("Default Output: {:?}", config);

        let engine = Engine {
            rx,
            host,
            config: config.clone(),
            device,
            command_rx,
        };

        let controller = EngineController {
            prod: tx,
            command_tx,
            sources: Arc::new(Mutex::new(Sources::new())),
            engine: Arc::new(Mutex::new(engine)),
            config: config.clone(),
        };

        Ok(controller)
    }

    pub fn start_stream(&self) -> Result<Stream, ()> {
        let rx_clone = self.rx.clone();
        println!("Starting stream...");

        let stream_result = match self.config.sample_format() {
            SampleFormat::I16 => self.device.build_output_stream(
                &self.config.config(),
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
                &self.config.config(),
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
                &self.config.config(),
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
}

pub struct EngineController {
    sources: Arc<Mutex<Sources>>,
    command_tx: Sender<EngineCommand>,
    prod: Sender<f32>,
    config: SupportedStreamConfig,
    engine: Arc<Mutex<Engine>>,
}

impl EngineController {
    pub fn add(&self, decoder: Symphonia) {
        let reader = SourceReader::new(decoder, self.config.clone());
        self.sources.lock().add_reader(reader);
    }

    pub fn open_source_reader(&self, path: String) {
        let source = Symphonia::new(path).unwrap();
        self.add(source);
    }

    pub async fn play(&self) {
        let sources = self.sources.clone();
        let sender = self.prod.clone();
        let engine = self.engine.clone();

        tokio::spawn(async move {
            let mut sources_lock = sources.lock();

            while let Some(sample) = sources_lock.next() {
                match sender.send(sample) {
                    Err(err) => eprintln!("Send Err: {}", err),
                    _ => (),
                }
            }
        });

        tokio::spawn(async move {
            let engine_lock = engine.lock();

            let stream = engine_lock.start_stream();

            loop {}
        });
    }
}

struct Sources {
    source_readers: Vec<SourceReader>,
    sources: Vec<Symphonia>,
}

impl Sources {
    pub fn new() -> Sources {
        Sources {
            sources: Default::default(),
            source_readers: Default::default(),
        }
    }

    pub fn add(&mut self, decoder: Symphonia) {
        self.sources.push(decoder);
    }

    pub fn add_reader(&mut self, reader: SourceReader) {
        self.source_readers.push(reader);
    }
}

impl Iterator for Sources {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        let mut samp = 0f32;
        let mut full_decoders = self.sources.len();

        for source in self.source_readers.iter_mut() {
            match source.next() {
                Some(sample) => {
                    samp += sample;
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
