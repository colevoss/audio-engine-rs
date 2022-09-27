struct SampleRate(pub u32);

impl SampleRate {
    // For now these conversions are happening to keep this as precise as possible
    fn ms_to_sample(&self, ms: u32) -> u32 {
        let samples_per_ms = self.0 as f32 / 1000 as f32;

        (samples_per_ms * ms as f32) as u32
    }

    fn sample_to_ms(&self, sample: u32) -> u32 {
        let ms_per_sample = sample as f32 / self.0 as f32;

        (ms_per_sample * 1000f32) as u32
    }
}

#[cfg(test)]
mod sample_rate_test {
    use crate::sample_rate::*;

    #[test]
    fn ms_to_sample_rate() {
        let sample_rate = SampleRate(44100);
        assert_eq!(sample_rate.ms_to_sample(2000), 88200);
        assert_eq!(sample_rate.ms_to_sample(1000), 44100);
        assert_eq!(sample_rate.ms_to_sample(100), 4410);
        assert_eq!(sample_rate.ms_to_sample(10), 441);
        assert_eq!(sample_rate.ms_to_sample(1), 44);
    }

    #[test]
    fn sample_to_ms() {
        let sample_rate = SampleRate(44100);

        assert_eq!(sample_rate.sample_to_ms(88200), 2000);
        assert_eq!(sample_rate.sample_to_ms(44100), 1000);
        assert_eq!(sample_rate.sample_to_ms(4410), 100);
        assert_eq!(sample_rate.sample_to_ms(441), 10);
        assert_eq!(sample_rate.sample_to_ms(44), 0);
        assert_eq!(sample_rate.sample_to_ms(4), 0);
    }
}
