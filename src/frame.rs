#[inline(always)]
pub fn new_frame(
    source_channel_count: usize,
    target_channel_count: usize,
) -> Box<dyn Frame + Send + Sync + 'static> {
    if source_channel_count == target_channel_count {
        return Box::new(OneToOneFrame::new(source_channel_count));
    } else if source_channel_count < target_channel_count {
        return Box::new(UpChannel::new(source_channel_count, target_channel_count));
    } else if source_channel_count > target_channel_count {
        return Box::new(DownChannel {});
    } else {
        panic!("What the hell happend")
    }
}

pub trait Frame {
    fn advance(&mut self);
    fn current_sample_index(&self) -> usize;
    fn current_channel_index(&self) -> usize;
    fn samples_advanced(&self) -> usize;
    fn reset(&mut self);
}

#[derive(Debug, Clone)]
pub struct OneToOneFrame {
    current_channel_index: usize,
    current_sample_index: usize, // Probably make this a u32
    channel_count: usize,
    samples_advanced: usize,
}

impl Frame for OneToOneFrame {
    fn reset(&mut self) {
        self.current_sample_index = 0;
        self.current_channel_index = 0;
    }

    // Advance the frame by one by advancing through each channel. When the
    // max number of channels has been reached, start at channel 1 (index 0)
    // and move to the next sample
    #[inline(always)]
    fn advance(&mut self) {
        self.samples_advanced += 1;
        if self.current_channel_index + 1 == self.channel_count {
            self.current_channel_index = 0;
            self.current_sample_index += 1;

            return;
        }

        self.current_channel_index += 1;
    }

    #[inline(always)]
    fn current_channel_index(&self) -> usize {
        self.current_channel_index
    }

    #[inline(always)]
    fn current_sample_index(&self) -> usize {
        self.current_sample_index
    }

    #[inline(always)]
    fn samples_advanced(&self) -> usize {
        self.samples_advanced
    }
}

impl OneToOneFrame {
    pub fn new(channel_count: usize) -> OneToOneFrame {
        OneToOneFrame {
            channel_count,
            current_channel_index: 0,
            current_sample_index: 0,
            samples_advanced: 0,
        }
    }
}

pub struct UpChannel {
    source_channel_count: usize,
    target_channel_count: usize,
    samples_advanced: usize,
}

impl UpChannel {
    pub fn new(source_channel_count: usize, target_channel_count: usize) -> Self {
        UpChannel {
            source_channel_count,
            target_channel_count,
            samples_advanced: 0,
        }
    }
}

impl Frame for UpChannel {
    #[inline]
    fn samples_advanced(&self) -> usize {
        self.samples_advanced
    }

    #[inline]
    fn current_sample_index(&self) -> usize {
        self.samples_advanced / self.target_channel_count
    }

    #[inline]
    fn current_channel_index(&self) -> usize {
        self.samples_advanced % self.source_channel_count
    }

    #[inline]
    fn advance(&mut self) {
        self.samples_advanced += 1;
    }

    #[inline]
    fn reset(&mut self) {
        self.samples_advanced = 0;
    }
}

struct DownChannel {}
impl DownChannel {}

impl Frame for DownChannel {
    fn advance(&mut self) {
        todo!()
    }

    fn current_sample_index(&self) -> usize {
        todo!()
    }

    fn current_channel_index(&self) -> usize {
        todo!()
    }

    fn samples_advanced(&self) -> usize {
        todo!()
    }

    fn reset(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::{new_frame, Frame, OneToOneFrame};
    #[test]
    fn frame_advacnes_single_channel() {
        let mut frame = new_frame(1, 1);
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 1);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 2);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 3);
    }

    #[test]
    fn frame_advances_two_channel() {
        let mut frame = OneToOneFrame::new(2);
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 1);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 1);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 1);
        assert_eq!(frame.current_sample_index(), 1);
    }

    #[test]
    fn frame_advances_three_channel() {
        let mut frame = OneToOneFrame::new(3);
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 1);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 2);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 1);
    }

    #[test]
    fn up_channel_one_to_two() {
        let mut frame = new_frame(1, 2);
        // Starting sample
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        // Should use starting sample again
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        // Should advance to second sample (index 1)
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 1);

        frame.advance();
        // Should stay on second sample (index 1)
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 1);

        frame.advance();
        // Should advance to third sample (index 2)
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 2);

        frame.advance();
        // Should stay on third sample (index 2)
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 2);
    }
}
#[cfg(test)]
mod frame_tests {
    use crate::frame::{new_frame, Frame, OneToOneFrame};
    #[test]
    fn frame_advacnes_single_channel() {
        let mut frame = new_frame(1, 1);
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 1);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 2);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 3);
    }

    #[test]
    fn frame_advances_two_channel() {
        let mut frame = OneToOneFrame::new(2);
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 1);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 1);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 1);
        assert_eq!(frame.current_sample_index(), 1);
    }

    #[test]
    fn frame_advances_three_channel() {
        let mut frame = OneToOneFrame::new(3);
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 1);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 2);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 1);
    }

    #[test]
    fn up_channel_one_to_two() {
        let mut frame = new_frame(1, 2);
        // Starting sample
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        // Should use starting sample again
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 0);

        frame.advance();
        // Should advance to second sample (index 1)
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 1);

        frame.advance();
        // Should stay on second sample (index 1)
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 1);

        frame.advance();
        // Should advance to third sample (index 2)
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 2);

        frame.advance();
        // Should stay on third sample (index 2)
        assert_eq!(frame.current_channel_index(), 0);
        assert_eq!(frame.current_sample_index(), 2);
    }
}
