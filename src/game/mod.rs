use audio_generator::*;
use audio_output::AudioOutput;
use bevy::prelude::*;

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
        let vco = Vco::new(
            SquareWave::default(),
            220.,
            Some(Attenuator::new(SquareWave::new(20.).as_raw(), 0.1).as_raw()),
        );
        let vca = Vca::new(
            as_raw_source(vco),
            as_raw_source(Envelope::new(0.2, 0.1, 0.2, 1.0)),
        );
        stream_handle.play_raw(as_raw_source(vca));
    }
}
