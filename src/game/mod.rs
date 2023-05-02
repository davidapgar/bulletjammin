use audio_generator::{as_raw_source, Envelope, RawSource, SquareWave, Vca};
use audio_output::AudioOutput;
use bevy::prelude::*;
use bevy::utils::Duration;
use rodio::source::Source;

pub mod audio_generator;
pub mod audio_output;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioOutput::default())
            .add_startup_system(audio_startup);
    }
}

fn audio_startup(mut audio_output: ResMut<AudioOutput>) {
    if let Some(stream_handle) = &audio_output.stream_handle {
        println!("here");
        let vca = Vca::new(
            as_raw_source(SquareWave::default()),
            as_raw_source(Envelope::new(0.5, 1.0)),
        );
        stream_handle.play_raw(as_raw_source(vca));
    }
}
