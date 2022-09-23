use cpal::{SampleFormat, SampleRate, SupportedBufferSize, SupportedStreamConfig};
use dawlib::builder::*;

fn main() {
    let stream_config = SupportedStreamConfig::new(
        2,
        SampleRate(44100),
        SupportedBufferSize::Unknown,
        SampleFormat::F32,
    );

    let mixer = MixerModel {
        channels: vec![
            ChannelModel {
                id: "chan-1".to_string(),
                clips: vec![
                    ClipModel {
                        path: "sounds/sample-3.wav".to_string(),
                        start_time_ms: 0,
                        duration_ms: 10,
                    },
                    // ClipModel {
                    //     path: "sounds/sample-3.wav".to_string(),
                    //     start_time_ms: 0,
                    //     duration_ms: 10,
                    // },
                    // ClipModel {
                    //     path: "sounds/sample-3.wav".to_string(),
                    //     start_time_ms: 0,
                    //     duration_ms: 10,
                    // },
                    // ClipModel {
                    //     path: "sounds/sample-3.wav".to_string(),
                    //     start_time_ms: 0,
                    //     duration_ms: 10,
                    // },
                    // ClipModel {
                    //     path: "sounds/sample-3.wav".to_string(),
                    //     start_time_ms: 0,
                    //     duration_ms: 10,
                    // },
                ],
            },
            ChannelModel {
                id: "chan-2".to_string(),
                clips: vec![ClipModel {
                    path: "sounds/sample-5.wav".to_string(),
                    start_time_ms: 0,
                    duration_ms: 10,
                }],
            },
            ChannelModel {
                id: "chan-3".to_string(),
                clips: vec![ClipModel {
                    path: "sounds/sample-1.wav".to_string(),
                    start_time_ms: 0,
                    duration_ms: 10,
                }],
            },
            ChannelModel {
                id: "chan-4".to_string(),
                clips: vec![ClipModel {
                    path: "sounds/sample-1.wav".to_string(),
                    start_time_ms: 0,
                    duration_ms: 10,
                }],
            },
        ],
    };

    let playback = PlaybackBuilder::new(&mixer, stream_config).unwrap();

    PlaybackBuilder::test(playback);
}
