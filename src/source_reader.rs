use crate::{
    frame::{new_frame, Frame},
    source::Source,
    symph::Symphonia,
};
use cpal::SupportedStreamConfig;
use rubato::{FftFixedInOut, FftFixedOut, Resampler};

pub struct SourceReader {
    source: Symphonia,
    resample_input_buf: Vec<Vec<f32>>,
    resample_output_buf: Vec<Vec<f32>>,
    // resampler: FftFixedInOut<f32>,
    resampler: FftFixedOut<f32>,
    target_channel_count: usize,
    source_channel_count: usize,
    frame: Box<dyn Frame + Send + Sync>,
}

// Currently working to convert from source channel count to target channel count
impl SourceReader {
    pub fn new(source: Symphonia, config: SupportedStreamConfig) -> Self {
        let target_sample_rate = config.sample_rate().0;
        let source_sample_rate = source.sample_rate().0;
        let target_channel_count = config.channels() as usize;
        let source_channel_count = source.channels();

        // println!("Sample Rates:");
        // println!(" Source: {}", source_sample_rate);
        // println!(" Target: {}", target_sample_rate);
        // println!("Channels:");
        // println!(" Source: {}", source_channel_count);
        // println!(" Target: {}", target_channel_count);

        // let resampler = FftFixedInOut::new(
        let resampler = FftFixedOut::new(
            source_sample_rate as usize,
            target_sample_rate as usize,
            2048,
            2,
            source_channel_count,
        )
        .unwrap();

        let input_buf = resampler.input_buffer_allocate();
        let output_buf = resampler.output_buffer_allocate();

        let mut reader = Self {
            frame: new_frame(source_channel_count, target_channel_count),
            source,
            resampler,
            resample_input_buf: input_buf,
            resample_output_buf: output_buf,
            target_channel_count,
            source_channel_count,
        };

        reader.refil();

        reader
    }

    // TODO: Put this in an iterator impl
    #[inline(always)]
    pub fn next(&mut self) -> Option<f32> {
        // return self.source.next();

        // This should work for both 1:1 and 1:(1+n) channel conversion
        // In an up channel setup (1 -> 2 channels) we need to make sure that the
        // samples advanced % 2 == 1 (odd) and the current sample index + 1 is the
        // same as the length of the last source channel buffer
        let could_be_last_sample = self.frame.samples_advanced() % self.target_channel_count != 0;
        let is_last_sample = self.resample_output_buf[self.frame.current_channel_index()].len()
            == self.frame.current_sample_index() + 1;

        if is_last_sample && could_be_last_sample {
            let read_samples = self.refil();

            if read_samples == 0 {
                return None;
            }
            self.frame.reset();
        }

        // Refigure sample index now that we have reset our position to 0
        // let sample_index = self.frame_position / self.target_channel_count;
        let samp = self.resample_output_buf[self.frame.current_channel_index()]
            [self.frame.current_sample_index()];

        self.frame.advance();
        Some(samp)
    }

    // IDEA: Each time we read a frame from the output buffer, could we also read one from the source and fill the input buffer?
    // Then when we have read all the frames from the output buffer, we could run a qucker resample.
    // I suppose this would keep a larger memory footprint
    #[inline(always)]
    pub fn refil(&mut self) -> usize {
        // How many frames do we need to get (samples * channels)
        let get_frame_count = self.resampler.input_frames_next();
        let mut n = 0;

        // Fill input buffer with samples
        'outer: while n < get_frame_count {
            // Get a sample for each channel
            for c in 0..self.source_channel_count {
                let sample = match self.source.next() {
                    Some(samp) => samp,
                    None => {
                        break 'outer;
                    }
                };

                self.resample_input_buf[c].push(sample);
            }

            n += 1;
        }

        // If we didn't read any, return 0
        if n == 0 {
            return 0;
        }

        // If we read less than the resampler expects, fill each buffer with 0's
        if n < get_frame_count {
            let mut n2 = n;
            while n2 < get_frame_count {
                for c in 0..self.source_channel_count {
                    self.resample_input_buf[c].push(0f32);
                }

                n2 += 1;
            }
        }

        if let Err(resample_err) = self.resampler.process_into_buffer(
            &self.resample_input_buf,
            &mut self.resample_output_buf,
            None,
        ) {
            eprintln!("Resamp error {}", resample_err);
        };

        // TODO: Can this just be done in another function
        for c in self.resample_input_buf.iter_mut() {
            c.clear();
        }

        return n;
    }
}
