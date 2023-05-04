use audio::audio_generator::*;
use audio::audio_output::AudioOutput;
use audio::Audio;
use bevy::prelude::*;

pub mod audio;
pub mod player;
pub mod world;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioOutput::default())
            .insert_resource(Audio::default())
            .add_system(button_system);
        app.add_plugin(audio::AudioPlugin)
            .add_plugin(world::WorldPlugin)
            .add_plugin(player::PlayerPlugin);
    }
}

fn button_system(keyboard_input: Res<Input<KeyCode>>, audio: ResMut<Audio>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        println!("Here");
        let vco = Vco::from_oscillator(SuperSaw::new(440.), 220.);
        let vca = Vca::new(vco, Envelope::new(0.2, 0.1, 0.2, 1.0));

        audio.play(vca.as_raw());
    }

    let mut keys = Vec::<u32>::new();

    if keyboard_input.just_pressed(KeyCode::Q) {
        keys.push(0);
    }
    if keyboard_input.just_pressed(KeyCode::W) {
        keys.push(2);
    }
    if keyboard_input.just_pressed(KeyCode::E) {
        keys.push(4);
    }
    if keyboard_input.just_pressed(KeyCode::R) {
        keys.push(5);
    }
    if keyboard_input.just_pressed(KeyCode::T) {
        keys.push(7);
    }

    for key in keys {
        let frequency = frequency_per_volt(key as f32 / 120.0 + 0.2);
        let vca = Vca::new(
            Vco::new(
                Vcf::new(SquareWave::new(frequency), frequency, 0.2),
                frequency / 2.,
                Envelope::new(0.3, 0.2, 0.05, 0.2),
            ),
            Envelope::new(0.3, 0.1, 0.05, 0.2),
        );
        audio.play(vca.as_raw());
    }
}
