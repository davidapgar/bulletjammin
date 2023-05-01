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
            //.add_startup_system(audio_startup);
            .add_startup_system(play_single_audio_startup_system);
    }
}

#[derive(TypeUuid)]
#[uuid = "F1FC2045-4C18-4845-979A-CCCFE557261C"]
pub struct SquareWave {
    samples: u32,
}

impl Source for SquareWave {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2
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

fn audio_startup() {}
