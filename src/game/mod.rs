use audio::Audio;
use audio_generator::*;
use audio_output::{play_queued_audio_system, AudioOutput};
use bevy::prelude::*;

pub mod audio;
pub mod audio_generator;
pub mod audio_output;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioOutput::default())
            .insert_resource(Audio::default())
            .add_startup_system(audio_startup)
            .add_system(play_queued_audio_system)
            .add_system(button_system);
    }
}

fn audio_startup(mut audio_output: ResMut<AudioOutput>) {
    if let Some(stream_handle) = &audio_output.stream_handle {
        println!("here");
        let vco = Vco::new(
            //SawWave::new(440.),
            SuperSaw::new(440.),
            220.,
            //Some(Attenuator::new(SawWave::new(10.).as_raw(), 0.1).as_raw()),
            None,
        );
        let vca = Vca::new(
            as_raw_source(vco),
            as_raw_source(Envelope::new(0.2, 0.1, 0.2, 1.0)),
        );
        stream_handle.play_raw(as_raw_source(vca));
    }
}

fn button_system(keyboard_input: Res<Input<KeyCode>>, audio: ResMut<Audio>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        println!("Here");
        let vco = Vco::new(
            SuperSaw::new(440.),
            220.,
            Some(Attenuator::new(SawWave::new(10.).as_raw(), 0.1).as_raw()),
        );
        let vca = Vca::new(
            as_raw_source(vco),
            as_raw_source(Envelope::new(0.2, 0.1, 0.2, 1.0)),
        );

        audio.play(as_raw_source(vca));
    }

    let mut keys = Vec::<u32>::new();

    if keyboard_input.just_pressed(KeyCode::Q) {
        keys.push(0);
    }
    if keyboard_input.just_pressed(KeyCode::W) {
        keys.push(1);
    }
    if keyboard_input.just_pressed(KeyCode::E) {
        keys.push(2);
    }
    if keyboard_input.just_pressed(KeyCode::R) {
        keys.push(3);
    }
    if keyboard_input.just_pressed(KeyCode::T) {
        keys.push(4);
    }

    for key in keys {
        let frequency = frequency_per_volt(key as f32 / 120.0 + 0.2);
        let vca = Vca::new(
            SawWave::new(frequency).as_raw(),
            Envelope::new(0.2, 0.1, 0.05, 0.2).as_raw(),
        );
        audio.play(vca.as_raw());
    }
}
