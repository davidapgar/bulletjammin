use rodio::source::Source;
use std::time::Duration;

/// Single channel audio generator
pub trait GenSource: Iterator<Item = f32> + Send + Sync + 'static {}

/// Type-erased wrapper around a source.
/// Has 1 channel, sample_rate of 44100
pub struct RawSource {
    source: Box<dyn GenSource>,
}

impl GenSource for RawSource {}

impl RawSource {
    pub fn new<T>(source: T) -> RawSource
    where
        T: GenSource,
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
    T: GenSource,
{
    RawSource::new(source)
}

const SAMPLE_RATE: f32 = 44100.0;

pub trait Oscillator: GenSource {
    fn set_frequency(&mut self, frequency: f32);
}

/// Frequency of C2 in hz. Base for 0.1 / octave (1v / octave, with -1.0 to 1.0 being -10 to 10).
const C2: f32 = 65.41;
/// Convert a "voltage" to a frequency
/// Scale of -1.0 to 1.0 as -10 to 10 volts
/// Each 0.1 is considered a "volt"
pub fn frequency_per_volt(volt: f32) -> f32 {
    C2 * 2.0_f32.powf(volt.clamp(-1.0, 1.0) * 10.0)
}

/// Convert a frequency into "voltage", then scale to -1.0..1.0
pub fn volts_per_frequency(frequency: f32) -> f32 {
    (frequency / C2).log2() / 10.0
}

pub struct Vco<T: Oscillator, CV: GenSource> {
    oscillator: T,
    base_voltage: f32,
    cv: Option<CV>,
    last_cv: f32,
}

impl<T, CV> GenSource for Vco<T, CV>
where
    T: Oscillator,
    CV: GenSource,
{
}

impl<T, CV> Vco<T, CV>
where
    T: Oscillator,
    CV: GenSource,
{
    pub fn new(oscillator: T, base_frequency: f32, cv: CV) -> Self {
        Self::new_internal(oscillator, base_frequency, Some(cv))
    }

    fn new_internal(mut oscillator: T, base_frequency: f32, cv: Option<CV>) -> Self {
        oscillator.set_frequency(base_frequency);
        let base_voltage = volts_per_frequency(base_frequency);

        Vco {
            oscillator,
            base_voltage,
            cv,
            last_cv: 0.0,
        }
    }

    pub fn as_raw(self) -> RawSource {
        RawSource::new(self)
    }
}

impl<T> Vco<T, RawSource>
where
    T: Oscillator,
{
    pub fn from_oscillator(oscillator: T, base_frequency: f32) -> Self {
        Vco::new_internal(oscillator, base_frequency, None)
    }
}

impl<T, CV> Iterator for Vco<T, CV>
where
    T: Oscillator,
    CV: GenSource,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cv) = &mut self.cv {
            if let Some(voltage) = cv.next() {
                if voltage != self.last_cv {
                    self.last_cv = voltage;
                    let frequency =
                        frequency_per_volt((voltage + self.base_voltage).clamp(-1., 1.));
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

impl GenSource for SquareWave {}

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

impl GenSource for SawWave {}

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

impl GenSource for RampWave {}

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

pub struct TriangleWave {
    frequency: f32,
    period: f32,
}

impl GenSource for TriangleWave {}

impl TriangleWave {
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

impl Oscillator for TriangleWave {
    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}

impl Iterator for TriangleWave {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let p_step = self.frequency / SAMPLE_RATE;

        self.period += p_step;
        if self.period > 1.0 {
            self.period -= 1.0;
        }

        // low to high to low
        // 0..0.5 is -0.5..0.5, 0.5..1 is 0.5..-0.5
        if self.period < 0.5 {
            // Scale to -0.5..0.5 (0..1 - 0.5)
            let sample = self.period * 2. - 0.5;
            Some(sample)
        } else {
            // >= 0.5..1.0
            // Scale to -0.5..0.5, invert
            let sample = (self.period - 0.5) * 2. - 0.5;
            Some(sample * -1.)
        }
    }
}

/// Noise from LFSR, based on the gameboy noise channel
/// 15 bit LFSR, sets 15 as bit 1 ^ bit 0 on shift
/// Scaled to -0.5 to 0.5
pub struct NoiseLFSR {
    frequency: f32,
    period: f32,
    lfsr: u16,
    last: u16,
}

impl GenSource for NoiseLFSR {}

impl NoiseLFSR {
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            period: 0.,
            lfsr: 1,
            last: 1,
        }
    }

    pub fn as_raw(self) -> RawSource {
        RawSource::new(self)
    }
}

impl Oscillator for NoiseLFSR {
    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}

impl Iterator for NoiseLFSR {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let p_step = self.frequency / SAMPLE_RATE;

        self.period += p_step;
        if self.period > 1.0 {
            self.period -= 1.0;
            // Update LFSR state
            let mut bit = self.lfsr & 0x01;
            self.last = bit;

            let mut lfsr = self.lfsr >> 1;
            bit = bit ^ (lfsr & 0x01);
            lfsr |= bit << 14;
            self.lfsr = lfsr;
        }

        if self.last == 0 {
            Some(-0.5)
        } else {
            Some(0.5)
        }
    }
}

pub struct SuperSaw {
    sub_oscillators: Vec<SawWave>,
}

impl GenSource for SuperSaw {}

impl SuperSaw {
    pub fn new(frequency: f32) -> Self {
        let mut sub_oscillators = Vec::<SawWave>::new();
        for _ in 0..8 {
            sub_oscillators.push(SawWave::new(frequency));
        }
        let mut super_saw = SuperSaw { sub_oscillators };

        super_saw.set_frequency(frequency);

        super_saw
    }
}

impl Oscillator for SuperSaw {
    fn set_frequency(&mut self, frequency: f32) {
        let mut offset = 0. - (self.sub_oscillators.len() / 2) as f32;
        for sub in &mut self.sub_oscillators {
            sub.set_frequency(frequency + offset);
            offset += 1.;
        }
    }
}

impl Iterator for SuperSaw {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut res = 0.0;
        for sub in &mut self.sub_oscillators {
            res += sub.next().unwrap();
        }
        Some(res)
    }
}

///! From [LP and HP Filter](https://www.musicdsp.org/en/latest/Filters/38-lp-and-hp-filter.html)
/// Frequency in Hz
/// Resonance is sqrt(2) (1.4142) to 0.1 low to high
pub struct Vcf<T: GenSource> {
    source: T,
    frequency: f32,
    resonance: f32,

    input: [f32; 2],
    output: [f32; 2],
    // Cached when frequency changes
    c: f32,
}

impl<T> GenSource for Vcf<T> where T: GenSource {}

impl<T> Vcf<T>
where
    T: GenSource,
{
    pub fn new(source: T, frequency: f32, resonance: f32) -> Self {
        let mut vcf = Self {
            source,
            frequency,
            resonance,
            input: [0., 0.],
            output: [0., 0.],
            c: 0.0,
        };

        vcf.set_frequency(frequency);
        vcf
    }

    pub fn as_raw(self) -> RawSource {
        RawSource::new(self)
    }
}

impl<T> Oscillator for Vcf<T>
where
    T: GenSource,
{
    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.c = 1.0 / (std::f32::consts::PI * frequency / SAMPLE_RATE as f32).tan();
    }
}

impl<T> Iterator for Vcf<T>
where
    T: GenSource,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(input) = self.source.next() else {
            return None;
        };

        let r = self.resonance;
        let c = self.c;
        let c2 = self.c * self.c;

        let a1 = 1.0 / (1.0 + (r * c) + c2);
        let a2 = 2.0 * a1;
        let a3 = a1;
        let b1 = 2.0 * (1.0 - c2) * a1;
        let b2 = (1.0 - (r * c) + c2) * a1;

        let output = (a1 * input) + (a2 * self.input[0]) + (a3 * self.input[1])
            - (b1 * self.output[0])
            - (b2 * self.output[1]);

        self.input[1] = self.input[0];
        self.input[0] = input;

        self.output[1] = self.output[0];
        self.output[0] = output;

        Some(output)
    }
}

pub struct Vca<S: GenSource, E: GenSource> {
    source: S,
    envelope: E,
}

impl<S, E> GenSource for Vca<S, E>
where
    S: GenSource,
    E: GenSource,
{
}

impl<S, E> Vca<S, E>
where
    S: GenSource,
    E: GenSource,
{
    pub fn new(source: S, envelope: E) -> Self {
        Vca { source, envelope }
    }

    pub fn as_raw(self) -> RawSource {
        RawSource::new(self)
    }
}

impl<S, E> Iterator for Vca<S, E>
where
    S: GenSource,
    E: GenSource,
{
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

impl GenSource for Envelope {}

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

    pub fn as_raw(self) -> RawSource {
        RawSource::new(self)
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

pub struct Attenuator<T: GenSource> {
    source: T,
    attenuation: f32,
}

impl<T> GenSource for Attenuator<T> where T: GenSource {}

impl<T> Attenuator<T>
where
    T: GenSource,
{
    pub fn new(source: T, attenuation: f32) -> Self {
        Attenuator {
            source,
            attenuation: attenuation.clamp(-1.0, 2.0),
        }
    }

    pub fn as_raw(self) -> RawSource {
        RawSource::new(self)
    }
}

impl<T> Iterator for Attenuator<T>
where
    T: GenSource,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.source.next() {
            Some(sample * self.attenuation)
        } else {
            None
        }
    }
}
