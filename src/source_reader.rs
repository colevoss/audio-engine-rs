use crate::{
    frame::{new_frame, Frame},
    source::Source,
    symph::Symphonia,
};
use cpal::SupportedStreamConfig;
use rubato::{FftFixedInOut, Resampler};

pub struct SourceReader {
    source: Symphonia,
    resample_input_buf: Vec<Vec<f32>>,
    resample_output_buf: Vec<Vec<f32>>,
    resampler: FftFixedInOut<f32>,
    frame_position: usize,
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

        println!("Sample Rates:");
        println!(" Source: {}", source_sample_rate);
        println!(" Target: {}", target_sample_rate);
        println!("Channels:");
        println!(" Source: {}", source_channel_count);
        println!(" Target: {}", target_channel_count);

        let resampler = FftFixedInOut::new(
            source_sample_rate as usize,
            target_sample_rate as usize,
            1024,
            source_channel_count,
        )
        .unwrap();

        let input_buf = resampler.input_buffer_allocate();
        let output_buf = resampler.output_buffer_allocate();

        let reader = Self {
            frame: new_frame(source_channel_count, target_channel_count),
            source,
            resampler,
            resample_input_buf: input_buf,
            resample_output_buf: output_buf,
            frame_position: 0,
            target_channel_count,
            source_channel_count,
        };

        reader
    }

    // TODO: Put this in an iterator impl
    pub fn next(&mut self) -> Option<f32> {
        if self.resample_output_buf[0].is_empty() {
            self.refil();
        }
        // How do we know we have used the last available sample??????????
        // In a 1:1 channel setup we need to check that the last channel in the output_buf
        // has the same length of (current sample index + 1)
        // Question: Can we use the same advanced-is-odd logic here???
        // if self.resample_output_buf[self.frame.current_channel_index()].len()
        //     == self.frame.current_sample_index() + 1
        // {
        //     // Refill and reset
        // }

        // |
        // V This should work for both 1:1 and 1:(1+n) channel conversion
        // In an up channel setup (1 -> 2 channels) we need to make sure that the
        // samples advanced % 2 == 1 (odd) and the current sample index + 1 is the
        // same as the length of the last source channel buffer
        let could_be_last_sample = self.frame.samples_advanced() % self.target_channel_count != 0;
        let is_last_sample = self.resample_output_buf[self.frame.current_channel_index()].len()
            == self.frame.current_sample_index() + 1;

        if is_last_sample && could_be_last_sample {
            // Refill
            let get_frame_count = self.resampler.input_frames_next();
            let mut n = 0;

            // Fill input buffer with samples
            while n < get_frame_count {
                // Get a sample for each channel
                for c in 0..self.source_channel_count {
                    let sample = match self.source.next() {
                        Some(samp) => samp,
                        None => break,
                    };

                    self.resample_input_buf[c].push(sample);
                }

                n += 1;
            }

            self.resampler.process_into_buffer(
                &self.resample_input_buf,
                &mut self.resample_output_buf,
                None,
            );

            // TODO: Can this just be done in another function
            for c in self.resample_input_buf.iter_mut() {
                c.clear();
            }

            // Reset frame position
            self.frame_position = 0;
            self.frame.reset();
            // sample_index = self.frame_position / self.target_channel_count;
        }

        // Refigure sample index now that we have reset our position to 0
        // let sample_index = self.frame_position / self.target_channel_count;
        let samp = self.resample_output_buf[self.frame.current_channel_index()]
            [self.frame.current_sample_index()];

        self.frame_position += 1;
        self.frame.advance();
        Some(samp)
    }

    // IDEA: Each time we read a frame from the output buffer, could we also read one from the source and fill the input buffer?
    // Then when we have read all the frames from the output buffer, we could run a qucker resample.
    // I suppose this would keep a larger memory footprint
    pub fn refil(&mut self) {
        // How many frames do we need to get (samples * channels)
        let get_frame_count = self.resampler.input_frames_next();
        let mut n = 0;

        // Fill input buffer with samples
        while n < get_frame_count {
            // Get a sample for each channel
            for c in 0..self.source_channel_count {
                let sample = match self.source.next() {
                    Some(samp) => samp,
                    None => {
                        println!("SDF");
                        break;
                    }
                };

                self.resample_input_buf[c].push(sample);
            }

            n += 1;
        }

        self.resampler.process_into_buffer(
            &self.resample_input_buf,
            &mut self.resample_output_buf,
            None,
        );

        // TODO: Can this just be done in another function
        for c in self.resample_input_buf.iter_mut() {
            c.clear();
        }
        // println!("CAP: {}", self.resample_input_buf[0].capacity());
    }
}
