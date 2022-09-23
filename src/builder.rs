use cpal::SupportedStreamConfig;

use crate::{source_reader::SourceReader, symph::Symphonia};

pub struct ClipModel {
    pub path: String,
    pub start_time_ms: i32,
    pub duration_ms: u32,
}

pub struct ChannelModel {
    pub id: String,
    // channel_count: usize, // Probalby don't need yet
    pub clips: Vec<ClipModel>,
}

pub struct MixerModel {
    pub channels: Vec<ChannelModel>,
}

pub struct PlaybackBuilder {}

pub struct Playback {
    channels: Vec<Channel>,
}

struct Channel {
    clips: Vec<SourceReader>,
}

impl PlaybackBuilder {
    // TODO: Make config a member of playback builder or something.
    pub fn new<'a>(mixer: &'a MixerModel, config: SupportedStreamConfig) -> Result<Playback, ()> {
        // Maybe use with_capacity
        let mut channels = Vec::<Channel>::with_capacity(mixer.channels.len());

        for chan in mixer.channels.iter() {
            let mut clips = Vec::<SourceReader>::with_capacity(chan.clips.len());

            for clip in chan.clips.iter() {
                let symp = Symphonia::new(clip.path.clone()).expect("Clip should have opened file");
                let reader = SourceReader::new(symp, config.clone());
                clips.push(reader);
            }

            channels.push(Channel { clips });
        }

        Ok(Playback { channels })
    }
}
