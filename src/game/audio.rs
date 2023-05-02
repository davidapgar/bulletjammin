use super::SquareWave;
use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use rodio::{source::Source, OutputStream, OutputStreamHandle, Sink};

/// Copy-pasta from bevy_audio.
///
/// ## Note
///
/// This will leak [`rodio::OutputStream`](rodio::OutputStream)
/// using [`std::mem::forget`].
/// This is to prevent it being dropped and stopping audio.
///
/// When used once, this is fine. Multiple times will leak memory.
#[derive(Resource)]
pub struct AudioOutput {
    pub(crate) stream_handle: Option<OutputStreamHandle>,
}

impl Default for AudioOutput {
    fn default() -> Self {
        if let Ok((stream, stream_handle)) = OutputStream::try_default() {
            // Leak `OutputStream` to prevent audio from stopping.
            std::mem::forget(stream);
            Self {
                stream_handle: Some(stream_handle),
            }
        } else {
            warn!("No audio device found.");
            Self {
                stream_handle: None,
            }
        }
    }
}

impl AudioOutput {}

#[derive(Resource, TypeUuid)]
#[uuid = "D6913CD1-1B92-46FB-8298-1974DB6A7CC4"]
pub struct AudioSink {
    sink: Sink,
}

pub fn play_queued_audio_system(
    audio_output: Res<AudioOutput>,
    audio_sources: Option<Res<Assets<SquareWave>>>,
    mut sinks: ResMut<Assets<AudioSink>>,
) {
}

pub fn play_single_audio_startup_system(mut audio_output: ResMut<AudioOutput>) {
    /*
    if let Some(stream_handle) = &audio_output.stream_handle {
        println!("here");
        let square_wave = SquareWave { samples: 0 };
        stream_handle.play_raw(square_wave);
    }
    */
}
