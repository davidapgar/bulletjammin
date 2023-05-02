use rodio::source::Source;
use std::time::Duration;

/// Type-erased wrapper around a source.
/// Has 1 channel, sample_rate of 44100
pub struct RawSource {
    source: Box<dyn Iterator<Item = f32> + Send>,
}

impl RawSource {
    pub fn new<T>(source: T) -> RawSource
    where
        T: Iterator<Item = f32> + Send + 'static,
    {
        RawSource {
            source: Box::new(source),
        }
    }
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
    RawSource::new(source)
}

const SAMPLE_RATE: f32 = 44100.0;

pub trait Oscillator {
    fn set_frequency(&mut self, frequency: f32);
}

/// Frequency of C2 in hz. Base for 0.1 / octave (1v / octave, with -1.0 to 1.0 being -10 to 10).
const C2: f32 = 65.41;
/// Convert a "voltage" to a frequency
/// Scale of -1.0 to 1.0 as -10 to 10 volts
/// Each 0.1 is considered a "volt"
fn frequency_per_volt(volt: f32) -> f32 {
    C2 * 2.0_f32.powf(volt.clamp(-1.0, 1.0) * 10.0)
}

/// Convert a frequency into "voltage", then scale to -1.0..1.0
fn volts_per_frequency(frequency: f32) -> f32 {
    (frequency / C2).log2() / 10.0
}

pub struct Vco<T: Oscillator + Iterator<Item = f32>> {
    oscillator: T,
    base_voltage: f32,
    cv: Option<RawSource>,
    last_cv: f32,
}

impl<T> Vco<T>
where
    T: Oscillator + Iterator<Item = f32>,
{
    pub fn new(mut oscillator: T, base_frequency: f32, cv: Option<RawSource>) -> Self {
        oscillator.set_frequency(base_frequency);
        let base_voltage = volts_per_frequency(base_frequency);

        Vco {
            oscillator,
            base_voltage,
            cv,
            last_cv: 0.0,
        }
    }
}

impl<T> Iterator for Vco<T>
where
    T: Oscillator + Iterator<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cv) = &mut self.cv {
            if let Some(voltage) = cv.next() {
                if voltage != self.last_cv {
                    self.last_cv = voltage;
                    let frequency = frequency_per_volt(voltage + self.base_voltage);
                    self.oscillator.set_frequency(frequency);
                }
            }
        }
        self.oscillator.next()
    }
}

pub struct SquareWave {
    /// Frequence of the square wave, in Hz
    frequency: f32,
    period: f32,
}

impl Oscillator for SquareWave {
    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}

impl SquareWave {
    pub fn new(frequency: f32) -> Self {
        SquareWave {
            frequency,
            period: 0.,
        }
    }

    pub fn as_raw(self) -> RawSource {
        RawSource::new(self)
    }
}

impl Default for SquareWave {
    fn default() -> Self {
        SquareWave {
            frequency: 440.,
            period: 0.,
        }
    }
}

impl Iterator for SquareWave {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // Calculate period for frequency and sample rate
        // period = 1sec / frequency, or sample_rate / frequency (for 44100, 221 = 200)
        // from that, get step length (to scale internal period from 0.0 to 1.0)
        // as 1 / period (eg, 1 / 200 = 0.005)
        // Add to current period. If > 1.0, -= 1.0.
        // if period < 0.5 high, else low
        //
        // so each sample requested, adjust period by 1 / sample_rate / frequency
        // or: p_step = frequency / sample_rate
        let p_step = self.frequency / SAMPLE_RATE;

        self.period += p_step;
        if self.period > 1.0 {
            self.period -= 1.0;
        }

        if self.period < 0.5 {
            Some(0.5)
        } else {
            Some(-0.5)
        }
    }
}

pub struct SawWave {
    frequency: f32,
    period: f32,
}

impl SawWave {
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            period: 0.,
        }
    }

    pub fn as_raw(self) -> RawSource {
        RawSource::new(self)
    }
}

impl Oscillator for SawWave {
    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}

impl Iterator for SawWave {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let p_step = self.frequency / SAMPLE_RATE;

        self.period += p_step;
        if self.period > 1.0 {
            self.period -= 1.0;
        }

        // Goes from 0.5 to -0.5 linearly
        Some((1.0 - self.period) - 0.5)
    }
}

pub struct RampWave {
    frequency: f32,
    period: f32,
}

impl RampWave {
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            period: 0.,
        }
    }

    pub fn as_raw(self) -> RawSource {
        RawSource::new(self)
    }
}

impl Oscillator for RampWave {
    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}

impl Iterator for RampWave {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let p_step = self.frequency / SAMPLE_RATE;

        self.period += p_step;
        if self.period > 1.0 {
            self.period -= 1.0;
        }

        // Goes from -0.5 to 0.5 linearly
        Some(self.period - 0.5)
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
    /// Time in seconds of attack
    attack: f32,
    /// Time in seconds of hold
    hold: f32,
    /// Time in seconds of release
    release: f32,
    /// Time that has elapsed.
    time: f32,
}

impl Envelope {
    pub fn new(amplitude: f32, attack: f32, hold: f32, release: f32) -> Self {
        Envelope {
            amplitude: amplitude.clamp(0.0, 1.0),
            attack,
            hold,
            release,
            time: 0.,
        }
    }
}

impl Iterator for Envelope {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        const STEP: f32 = 1. / 44100.;
        let time = self.time;
        self.time += STEP;

        if self.time < self.attack {
            let res = self.amplitude * (time / self.attack);
            Some(res)
        } else if self.time < self.attack + self.hold {
            Some(self.amplitude)
        } else if self.time < self.attack + self.hold + self.release {
            let res = self.amplitude * (1.0 - ((time - (self.attack + self.hold)) / self.release));
            Some(res)
        } else {
            self.time = time;
            None
        }
    }
}

pub struct Attenuator {
    source: RawSource,
    attenuation: f32,
}

impl Attenuator {
    pub fn new(source: RawSource, attenuation: f32) -> Self {
        Attenuator {
            source,
            attenuation: attenuation.clamp(-1.0, 1.0),
        }
    }

    pub fn as_raw(self) -> RawSource {
        RawSource::new(self)
    }
}

impl Iterator for Attenuator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.source.next() {
            Some(sample * self.attenuation)
        } else {
            None
        }
    }
}
