use std::sync::Arc;

use cpal::{SampleFormat, SampleRate, SupportedBufferSize, SupportedStreamConfig};
use dawlib::builder::*;
use dawlib::engine::EngineController;
use parking_lot::lock_api::Mutex;

fn main() {
    let stream_config = SupportedStreamConfig::new(
        2,
        SampleRate(44100),
        SupportedBufferSize::Unknown,
        SampleFormat::F32,
    );

    let mut channels = Vec::<ChannelModel>::new();

    for _ in 0..1 {
        channels.push(ChannelModel {
            id: "chan-1".to_string(),
            clips: vec![ClipModel {
                path: "sounds/sample-1.wav".to_string(),
                start_time_ms: 0,
                duration_ms: 10,
            }],
        });
        channels.push(ChannelModel {
            id: "chan-2".to_string(),
            clips: vec![ClipModel {
                path: "sounds/sample-2.wav".to_string(),
                start_time_ms: 0,
                duration_ms: 10,
            }],
        });
        channels.push(ChannelModel {
            id: "chan-3".to_string(),
            clips: vec![ClipModel {
                path: "sounds/sample-3.wav".to_string(),
                start_time_ms: 0,
                duration_ms: 10,
            }],
        });
        channels.push(ChannelModel {
            id: "chan-4".to_string(),
            clips: vec![ClipModel {
                path: "sounds/sample-4.wav".to_string(),
                start_time_ms: 0,
                duration_ms: 10,
            }],
        });
        channels.push(ChannelModel {
            id: "chan-4".to_string(),
            clips: vec![ClipModel {
                path: "sounds/sample-5.wav".to_string(),
                start_time_ms: 0,
                duration_ms: 10,
            }],
        });
    }

    // for _ in 0..10 {
    //     channels.push(ChannelModel {
    //         id: "chan-2".to_string(),
    //         clips: vec![ClipModel {
    //             path: "sounds/sample-1.wav".to_string(),
    //             start_time_ms: 0,
    //             duration_ms: 10,
    //         }],
    //     })
    // }

    // for _ in 0..10 {
    //     channels.push(ChannelModel {
    //         id: "chan-2".to_string(),
    //         clips: vec![ClipModel {
    //             path: "sounds/sample-3.wav".to_string(),
    //             start_time_ms: 0,
    //             duration_ms: 10,
    //         }],
    //     })
    // }

    let mixer = MixerModel { channels };

    println!("Creating thing");
    let playback = PlaybackBuilder::new(&mixer, stream_config.config()).unwrap();
    println!("Thing Created");

    let controller = EngineController::new().unwrap();
    let arc_controller = Arc::new(Mutex::new(controller));

    PlaybackBuilder::test(playback, arc_controller);
}
