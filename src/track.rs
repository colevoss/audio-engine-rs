use crossbeam::channel::Receiver;
use parking_lot::Mutex;
use std::{sync::Arc, thread};

use crate::{
    engine::{Engine, EngineController},
    source_reader::SourceReader,
};

pub struct Track {
    source_readers: Vec<SourceReader>,
    source: SourceReader,
}

impl Track {
    pub fn new(reader: SourceReader) -> Self {
        Track {
            source_readers: Default::default(),
            source: reader,
        }
    }

    pub fn add(&mut self, reader: SourceReader) {
        self.source_readers.push(reader);
    }

    #[inline]
    pub fn next_sample(&mut self) -> Option<f32> {
        self.source.next()
    }

    pub fn test() {}
}
