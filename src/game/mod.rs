use animation::AnimationPlugin;
use audio::audio_generator::*;
use audio::audio_output::AudioOutput;
use audio::Audio;
use bevy::prelude::*;

pub mod animation;
pub mod assets;
pub mod audio;
pub mod cannon;
pub mod enemy;
pub mod menu;
pub mod player;
pub mod song;
pub mod world;

pub struct Plugin;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

#[derive(Resource)]
enum EndState {
    GameOver,
    Winner,
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .insert_resource(AudioOutput::default())
            .insert_resource(Audio::default())
            .add_startup_system(spawn_camera)
            .add_system(button_system);
        app.add_plugin(menu::MenuPlugin)
            .add_plugin(assets::AssetPlugin)
            .add_plugin(animation::AnimationPlugin)
            .add_plugin(audio::AudioPlugin)
            .add_plugin(cannon::CannonPlugin)
            .add_plugin(enemy::EnemyPlugin)
            .add_plugin(player::PlayerPlugin)
            .add_plugin(world::WorldPlugin);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn button_system(keyboard_input: Res<Input<KeyCode>>, audio: ResMut<Audio>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        println!("Here");
        let vco = Vco::from_oscillator(SuperSaw::new(440.), 220.);
        let vca = Vca::new(vco, Envelope::new(0.2, 0.1, 0.2, 1.0));

        audio.play(vca.as_raw());
    }

    let mut keys = Vec::<u32>::new();

    if keyboard_input.just_pressed(KeyCode::Z) {
        keys.push(0);
    }
    if keyboard_input.just_pressed(KeyCode::X) {
        keys.push(2);
    }
    if keyboard_input.just_pressed(KeyCode::C) {
        keys.push(4);
    }
    if keyboard_input.just_pressed(KeyCode::V) {
        keys.push(5);
    }
    if keyboard_input.just_pressed(KeyCode::B) {
        keys.push(7);
    }

    for key in keys {
        let frequency = frequency_per_volt(key as f32 / 120.0 + 0.2);
        let vca = Vca::new(
            Vco::new(
                //Vcf::new(SquareWave::new(frequency), frequency, 1.41),
                Vcf::new(SquareWave::new(frequency), frequency, 0.1),
                frequency / 2.,
                Envelope::new(0.4, 0.2, 0.05, 0.2),
            ),
            Envelope::new(0.3, 0.1, 0.05, 0.2),
        );
        audio.play(vca.as_raw());
        //let osc = Attenuator::new(TriangleWave::new(frequency), 2.0);

        /* Kick
        let frequency = frequency_per_volt(key as f32 / 120.0 + 0.0);

        let kick_env = Envelope::new(1.0, 0.001, 0.1, 0.3);
        let freq_env = Envelope::new(0.02, 0.0, 0.0, 0.2);
        let vco = Vco::new(TriangleWave::new(frequency), frequency, freq_env);
        let osc = Attenuator::new(vco, 2.0);
        let vca = Vca::new(osc, kick_env);
        */

        /* Snare
        let snare_env = Envelope::new(0.5, 0.001, 0.0, 0.3);
        let vco = NoiseLFSR::new(20000.);
        let osc = vco;
        let vca = Vca::new(osc, snare_env);
        audio.play(vca.as_raw());
        */
    }
}
