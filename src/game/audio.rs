use super::audio_generator::RawSource;
use bevy::prelude::*;
use std::collections::VecDeque;
use std::sync::RwLock;

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
