use cpal::{
    ChannelCount, SampleFormat, SampleRate, SupportedBufferSize, SupportedOutputConfigs,
    SupportedStreamConfig,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dawlib::builder::{ChannelModel, ClipModel, MixerModel, PlaybackBuilder};

fn build_playback_benchmark(c: &mut Criterion) {
    let stream_config = SupportedStreamConfig::new(
        2,
        SampleRate(44100),
        SupportedBufferSize::Unknown,
        SampleFormat::F32,
    );
    // let mixer = black_box(MixerModel {
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
                    ClipModel {
                        path: "sounds/sample-3.wav".to_string(),
                        start_time_ms: 0,
                        duration_ms: 10,
                    },
                    ClipModel {
                        path: "sounds/sample-3.wav".to_string(),
                        start_time_ms: 0,
                        duration_ms: 10,
                    },
                    ClipModel {
                        path: "sounds/sample-3.wav".to_string(),
                        start_time_ms: 0,
                        duration_ms: 10,
                    },
                    ClipModel {
                        path: "sounds/sample-3.wav".to_string(),
                        start_time_ms: 0,
                        duration_ms: 10,
                    },
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
        // });
    };

    c.bench_function("PlaybackBuilder", |b| {
        b.iter(|| PlaybackBuilder::new(&mixer, stream_config.clone()))
    });
}

criterion_group!(benchmarks, build_playback_benchmark);
criterion_main!(benchmarks);
