use self::audio_generator::*;
use self::audio_output::{play_queued_audio_system, AudioOutput};
use bevy::prelude::*;
use std::collections::VecDeque;
use std::sync::RwLock;

pub mod audio_generator;
pub mod audio_output;

pub struct AudioPlugin;

impl bevy::app::Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioOutput::default())
            .insert_resource(Audio::default())
            .add_startup_system(audio_startup)
            .add_system(play_queued_audio_system);
    }
}

fn audio_startup(audio_output: ResMut<AudioOutput>) {
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
        stream_handle.play_raw(as_raw_source(vca)).unwrap();
    }
}

#[derive(Resource)]
pub struct Audio {
    pub(crate) queue: RwLock<VecDeque<RawSource>>,
}

impl Default for Audio {
    fn default() -> Self {
        Self {
            queue: Default::default(),
        }
    }
}

impl Audio {
    pub fn play(&self, source: RawSource) {
        self.queue.write().unwrap().push_back(source);
    }
}
