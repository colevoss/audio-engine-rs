use crossbeam::channel::{Receiver, Sender};

// Holds a list of Tracks
// Mixes all channels together
pub struct Mixer {}

// Holds a list of clips
// Holds a list of Processors
// Is in charge of channeling source data through its Processors/
// to the mixer
pub struct Track {
    clips: Vec<Source>,
    processors: Vec<Box<dyn Processor>>,
    rx: Receiver<f32>,
    tx: Sender<f32>,
}

impl Track {
    pub fn next(&mut self) -> f32 {
        let mut sample = self.clips.get_mut(0).unwrap().next().unwrap();
        let mut proc_iter = self.processors.iter();

        while let Some(processor) = proc_iter.next() {
            sample = processor.process(sample);
        }

        sample
    }
}

// Can be anything, like resampler, volume, pan, or other pugins
pub trait Processor {
    /// Process one sample at a time.
    /// This might need some sort of state argument variable
    fn process(&self, sample: f32) -> f32;

    // fn process_buffer(&self, buffer: &mut [T]);
}

// For now this can be a clip
pub struct Source {}

impl Iterator for Source {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(1f32)
    }
}
