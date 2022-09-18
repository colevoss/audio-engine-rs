use std::fs::File;
// use std::path::Path;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer, SignalSpec};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units;

use crate::source::Source;

pub struct Symphonia {
    current_frame: usize,
    buffer: SampleBuffer<f32>,
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    spec: SignalSpec,
}

impl Symphonia {
    pub fn new(path: String) -> Result<Symphonia, ()> {
        let file = Box::new(File::open(path).unwrap());
        let mss = MediaSourceStream::new(file, Default::default());
        let hint = Hint::new();

        let mut format_opts: FormatOptions = Default::default();
        format_opts.enable_gapless = true;
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .expect("Cannot get probe");

        let mut format = probed.format;
        let track = match format.default_track() {
            Some(track) => track,
            None => return Err(()),
        };
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .unwrap();

        let decoded = loop {
            let packet = format.next_packet().unwrap();
            match decoder.decode(&packet) {
                Ok(decoded) => break decoded,
                Err(_) => return Err(()),
            }
        };

        let spec = decoded.spec().to_owned();
        let buffer = Self::get_new_buffer(decoded, &spec);

        return Ok(Symphonia {
            buffer,
            decoder,
            format,
            spec,
            current_frame: 0,
        });
    }

    fn get_new_buffer(decoded: AudioBufferRef, spec: &SignalSpec) -> SampleBuffer<f32> {
        let duration = units::Duration::from(decoded.capacity() as u64);
        let buffer = SampleBuffer::<f32>::new(duration, spec.clone());
        buffer
    }

    // fn read_bytes(&mut self, byte_count: usize) {
    //     let read_sample_count = 0;
    //     self.buffer.samples()[read_sample_count..byte_count]
    // }
}

impl Source for Symphonia {
    fn channels(&self) -> usize {
        self.spec.channels.count() as usize
    }

    fn sample_rate(&self) -> cpal::SampleRate {
        cpal::SampleRate(self.spec.rate)
    }
}

impl Iterator for Symphonia {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        if self.current_frame == self.buffer.len() {
            let decoded = loop {
                let packet = match self.format.next_packet() {
                    Ok(packet) => packet,
                    Err(Error::IoError(err)) => {
                        if err.kind() == std::io::ErrorKind::UnexpectedEof {
                            return None;
                        }
                        println!("IOError: {}", err.to_string());
                        return None;
                    }
                    Err(err) => {
                        println!("Err: {}", err.to_string());
                        return None;
                    }
                };

                match self.decoder.decode(&packet) {
                    Ok(decoded) => break decoded,
                    Err(_) => return None,
                }
            };

            self.buffer.copy_interleaved_ref(decoded);
            self.current_frame = 0;
        }

        let sample = self.buffer.samples()[self.current_frame];
        self.current_frame += 1;

        Some(sample)
    }
}
