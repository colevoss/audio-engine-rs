use std::io::{Read, Seek};

use hound::{SampleFormat, WavReader, WavSpec};

pub trait Source: Iterator
where
    Self::Item: cpal::Sample,
{
    fn channels(&self) -> usize;
    fn sample_rate(&self) -> cpal::SampleRate;

    fn seek(&self) -> Result<(), ()> {
        todo!()
    }
}

pub struct HoundWav<R>
where
    R: Read + Seek,
{
    reader: WavReader<R>,
    spec: WavSpec,
}

impl<R> HoundWav<R>
where
    R: Read + Seek,
{
    pub fn open(path: R) -> Result<HoundWav<R>, ()> {
        let reader = WavReader::new(path).unwrap();
        let spec = reader.spec();

        Ok(HoundWav { reader, spec })
    }
}

impl<R> Source for HoundWav<R>
where
    R: Read + Seek,
{
    fn channels(&self) -> usize {
        self.spec.channels as usize
    }

    fn sample_rate(&self) -> cpal::SampleRate {
        cpal::SampleRate(self.spec.sample_rate)
    }
}

impl<R> Iterator for HoundWav<R>
where
    R: Read + Seek,
{
    type Item = i16;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let sample = match (self.spec.sample_format, self.spec.bits_per_sample) {
            (SampleFormat::Int, 8) => self
                .reader
                .samples()
                .next()
                .map(|v| i8_to_i16(v.unwrap_or(0))),
            (SampleFormat::Int, 16) => self.reader.samples().next().map(|v| v.unwrap_or(0)),
            (SampleFormat::Int, 24) => self
                .reader
                .samples()
                .next()
                .map(|v| i24_to_i16(v.unwrap_or(0))),
            (SampleFormat::Int, 32) => self
                .reader
                .samples()
                .next()
                .map(|v| i32_to_i16(v.unwrap_or(0))),
            (SampleFormat::Float, 32) => self
                .reader
                .samples()
                .next()
                .map(|v| f32_to_i16(v.unwrap_or(0f32))),
            (_, _) => panic!("BAD"),
        };

        sample
    }
}

/// Returns a 32 bit WAV float as an i16. WAV floats are typically in the range of
/// [-1.0, 1.0] while i16s are in the range [-32768, 32767]. Note that this
/// function definitely causes precision loss but hopefully this isn't too
/// audiable when actually playing?
fn f32_to_i16(f: f32) -> i16 {
    // prefer to clip the input rather than be excessively loud.
    (f.max(-1.0).min(1.0) * i16::max_value() as f32) as i16
}

/// Returns an 8-bit WAV int as an i16. This scales the sample value by a factor
/// of 256.
fn i8_to_i16(i: i8) -> i16 {
    i as i16 * 256
}

/// Returns a 24 bit WAV int as an i16. Note that this is a 24 bit integer, not a
/// 32 bit one. 24 bit ints are in the range [−8,388,608, 8,388,607] while i16s
/// are in the range [-32768, 32767]. Note that this function definitely causes
/// precision loss but hopefully this isn't too audiable when actually playing?
fn i24_to_i16(i: i32) -> i16 {
    (i >> 8) as i16
}

/// Returns a 32 bit WAV int as an i16. 32 bit ints are in the range
/// [-2,147,483,648, 2,147,483,647] while i16s are in the range [-32768, 32767].
/// Note that this function definitely causes precision loss but hopefully this
/// isn't too audiable when actually playing?
fn i32_to_i16(i: i32) -> i16 {
    (i >> 16) as i16
}
