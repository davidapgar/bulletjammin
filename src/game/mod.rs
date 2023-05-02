use audio::{play_single_audio_startup_system, AudioOutput};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::utils::Duration;
use rodio::source::Source;

pub mod audio;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioOutput::default())
            .add_startup_system(audio_startup);
    }
}

#[derive(TypeUuid)]
#[uuid = "F1FC2045-4C18-4845-979A-CCCFE557261C"]
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

struct AnySource {
    source: Box<dyn Source<Item = f32> + Send>,
}

impl Source for AnySource {
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

impl Iterator for AnySource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next()
    }
}

fn as_any_source<T>(source: T) -> AnySource
where
    T: Source<Item = f32> + Send + 'static,
{
    AnySource {
        source: Box::new(source),
    }
}

/// Type-erased wrapper around a source.
/// Has 1 channel, sample_rate of 44100
struct RawSource {
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

fn as_raw_source<T>(source: T) -> RawSource
where
    T: Iterator<Item = f32> + Send + 'static,
{
    RawSource {
        source: Box::new(source),
    }
}

struct Vca {
    source: RawSource,
    envelope: RawSource,
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

struct Envelope {
    /// Maximum amplitude of the envelope, from 0.0 to 1.0
    amplitude: f32,
    /// Length of the envelope, in seconds.
    length: f32,
    /// Running time of the envelope
    time: f32,
}

impl Envelope {
    fn new(amplitude: f32, length: f32) -> Self {
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

fn audio_startup(mut audio_output: ResMut<AudioOutput>) {
    if let Some(stream_handle) = &audio_output.stream_handle {
        println!("here");
        let vca = Vca {
            source: as_raw_source(SquareWave::default()),
            envelope: as_raw_source(Envelope::new(0.5, 1.0)),
        };
        stream_handle.play_raw(as_any_source(as_raw_source(vca)));
    }
}
