use crossbeam::channel::{self, Receiver, Sender};
use parking_lot::Mutex;
use std::{
    sync::{mpsc::channel, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};

use cpal::{StreamConfig, SupportedStreamConfig};

use crate::{engine::EngineController, source_reader::SourceReader, symph::Symphonia};

#[derive(Clone, Debug)]
pub struct ClipModel {
    pub path: String,
    pub start_time_ms: i32,
    pub duration_ms: u32,
}

#[derive(Clone, Debug)]
pub struct ChannelModel {
    pub id: String,
    // channel_count: usize, // Probalby don't need yet
    pub clips: Vec<ClipModel>,
}

#[derive(Clone, Debug)]
pub struct MixerModel {
    pub channels: Vec<ChannelModel>,
}

pub struct PlaybackBuilder {}

pub struct Playback {
    channels: Vec<Channel>,
    config: StreamConfig,
}

pub struct PlayableClip {
    reader: SourceReader,
    clip_model: ClipModel,
}

struct Channel {
    // clips: Vec<SourceReader>,
    clips: Vec<PlayableClip>,
}

impl Channel {}

impl PlaybackBuilder {
    // TODO: Make config a member of playback builder or something.
    pub fn new<'a>(mixer: &'a MixerModel, config: StreamConfig) -> Result<Playback, ()> {
        // Maybe use with_capacity
        let mut channels = Vec::<Channel>::with_capacity(mixer.channels.len());

        // Start a new thread for each channel
        for chan in mixer.channels.iter() {
            let mut clips = Vec::<PlayableClip>::with_capacity(chan.clips.len());

            for clip in chan.clips.iter() {
                let symp = Symphonia::new(clip.path.clone()).expect("Clip should have opened file");
                let reader = SourceReader::new(symp, config.clone());

                let playable_clip = PlayableClip {
                    reader,
                    clip_model: clip.clone(),
                };

                clips.push(playable_clip);
            }

            channels.push(Channel { clips });
        }

        // Ok(Playback { channels })
        Ok(Playback { channels, config })
    }

    pub fn test(playback: Playback, engine_controller: Arc<Mutex<EngineController>>) {
        let mut thread_handlers = Vec::new();

        let mut channel_receivers = Vec::<Receiver<f32>>::with_capacity(playback.channels.len());
        let (mixer_tx, mixer_rx) = channel::bounded::<f32>(playback.channels.len());
        println!("BFS {:?}", playback.config.buffer_size);
        let buffer_size = match playback.config.buffer_size {
            cpal::BufferSize::Fixed(buffer_size) => buffer_size as usize,
            cpal::BufferSize::Default => 1024,
        };

        println!("Buffer Size {}", buffer_size);

        // TODO: Only play channels with clips
        // TODO: Only play channels with clips after the current play head
        for channel in playback.channels {
            let (channel_tx, channel_rx) = channel::bounded::<f32>(buffer_size * 2);
            channel_receivers.push(channel_rx);

            let t = thread::spawn(move || {
                let mut clips = channel.clips;
                println!("Spawned!");
                // let clip_count = channel.clips.len();
                if let Some(clip) = clips.get_mut(0) {
                    while let Some(sample) = clip.reader.next() {
                        // println!("Got sample");
                        match channel_tx.send(sample) {
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                            _ => {}
                        }
                    }

                    loop {
                        channel_tx.send(0f32).unwrap();
                    }
                }
            });

            thread_handlers.push(t);
        }

        let engine = engine_controller.clone();

        let mixer_thread = thread::spawn(move || {
            println!("WTF");
            let engine_lock = engine.lock();
            engine_lock.play();
            loop {
                let mut sample_total = 0f32;
                // For now using this to start the stream

                // For each channel's sender, get the next sample. These should all be
                // in sync
                for channel_receiver in channel_receivers.iter_mut() {
                    match channel_receiver.recv() {
                        Ok(sample) => {
                            // println!("{}", sample * 10000000f32);
                            // println!("{}", sample);
                            sample_total += sample;
                        }
                        Err(e) => {
                            eprintln!("BAD THINGS {}", e)
                        }
                    }
                }

                // println!("Sample total {}", sample_total);

                engine_lock
                    .prod
                    // .send(sample_total / 50f32)
                    .send(sample_total)
                    .expect("Should have seft");
                // mixer_tx.send(sample_total).expect("Should have sent");
                sample_total = 0f32;
            }
        });

        // while let Ok(mixed_sample) = mixer_rx.recv() {
        //     // println!("Mixed: {}", mixed_sample * 10000000f32);
        //     println!("Mixed: {}", mixed_sample);
        // }

        for t in thread_handlers {
            t.join().unwrap();
        }

        mixer_thread.join().unwrap();
    }
}
