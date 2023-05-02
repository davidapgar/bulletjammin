use rodio::source::Source;
use std::time::Duration;

/// Type-erased wrapper around a source.
/// Has 1 channel, sample_rate of 44100
pub struct RawSource {
    source: Box<dyn Iterator<Item = f32> + Send>,
}

impl Source for RawSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for RawSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next()
    }
}

pub fn as_raw_source<T>(source: T) -> RawSource
where
    T: Iterator<Item = f32> + Send + 'static,
{
    RawSource {
        source: Box::new(source),
    }
}

pub struct SquareWave {
    samples: u32,
}

impl Default for SquareWave {
    fn default() -> Self {
        SquareWave { samples: 0 }
    }
}

impl Source for SquareWave {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for SquareWave {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.samples = self.samples.wrapping_add(1);
        if self.samples > 100 {
            self.samples = 0;
        }

        if self.samples < 50 {
            Some(-0.2)
        } else {
            Some(0.2)
        }
    }
}

pub struct Vca {
    source: RawSource,
    envelope: RawSource,
}

impl Vca {
    pub fn new(source: RawSource, envelope: RawSource) -> Self {
        Vca { source, envelope }
    }
}

impl Iterator for Vca {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(sample), Some(env)) = (self.source.next(), self.envelope.next()) {
            if env > 0. {
                Some(sample * env)
            } else {
                Some(0.0)
            }
        } else {
            None
        }
    }
}

pub struct Envelope {
    /// Maximum amplitude of the envelope, from 0.0 to 1.0
    amplitude: f32,
    /// Length of the envelope, in seconds.
    length: f32,
    /// Running time of the envelope
    time: f32,
}

impl Envelope {
    pub fn new(amplitude: f32, length: f32) -> Self {
        Envelope {
            amplitude: amplitude.clamp(0.0, 1.0),
            length,
            time: 0.,
        }
    }
}

impl Iterator for Envelope {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        const step: f32 = 1. / 44100.;

        if self.time > self.length {
            None
        } else {
            let res = self.amplitude * (self.time / self.length);
            self.time += step;
            Some(res)
        }
    }
}
