use crossbeam::channel::{self, Receiver, Sender};
use std::{
    sync::mpsc::channel,
    thread::{self, JoinHandle},
    time::Duration,
};

use cpal::SupportedStreamConfig;

use crate::{source_reader::SourceReader, symph::Symphonia};

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
    pub fn new<'a>(mixer: &'a MixerModel, config: SupportedStreamConfig) -> Result<Playback, ()> {
        // Maybe use with_capacity
        let mut channels = Vec::<Channel>::with_capacity(mixer.channels.len());

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
        Ok(Playback { channels })
    }

    pub fn test(playback: Playback) {
        let mut thread_handlers = Vec::new();

        let mut channel_receivers = Vec::<Receiver<f32>>::with_capacity(playback.channels.len());
        let (mixer_tx, mixer_rx) = channel::bounded::<f32>(playback.channels.len());

        // TODO: Only play channels with clips
        // TODO: Only play channels with clips after the current play head
        for channel in playback.channels {
            let (channel_tx, channel_rx) = channel::bounded::<f32>(channel.clips.len());
            channel_receivers.push(channel_rx);

            let t = thread::spawn(move || {
                let mut clips = channel.clips;
                // let clip_count = channel.clips.len();
                if let Some(clip) = clips.get_mut(0) {
                    while let Some(sample) = clip.reader.next() {
                        thread::sleep(Duration::from_secs(1));
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

        let mixer_thread = thread::spawn(move || loop {
            let mut sample_total = 0f32;
            let mut i = 0;

            for channel_receiver in channel_receivers.iter_mut() {
                match channel_receiver.recv() {
                    Ok(sample) => {
                        println!("{}", sample * 10000000f32);
                        sample_total += sample;
                        i += 1;
                    }
                    Err(e) => {
                        eprintln!("BAD THINGS {}", e)
                    }
                }
            }

            mixer_tx.send(sample_total).expect("Should have sent");
            sample_total = 0f32;
            i = 0;
        });

        while let Ok(mixed_sample) = mixer_rx.recv() {
            println!("Mixed: {}", mixed_sample * 10000000f32);
        }

        for t in thread_handlers {
            t.join().unwrap();
        }

        mixer_thread.join().unwrap();
    }
}
