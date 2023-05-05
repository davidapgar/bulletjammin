use bevy::prelude::*;
use std::collections::HashMap;

pub struct AnimationPlugin;

impl bevy::app::Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animation_system);
    }
}

pub struct AnimationFrame {
    idx: usize,
    length: f32,
}

impl AnimationFrame {
    pub fn new(idx: usize, length: f32) -> Self {
        AnimationFrame { idx, length }
    }
}

#[derive(Component)]
pub struct Animation {
    frames: Vec<AnimationFrame>,
    repeat: bool,

    frame: usize,
    running: bool,
    timer: Timer,
}

impl Animation {
    pub fn new(frames: Vec<AnimationFrame>, repeat: bool) -> Animation {
        Self {
            frames,
            repeat,

            frame: 0,
            running: true,
            timer: Timer::new(bevy::utils::Duration::from_nanos(1), TimerMode::Once),
        }
    }

    fn next_frame(&mut self) -> Option<&AnimationFrame> {
        if !self.running {
            return None;
        }
        self.frame += 1;

        if self.frame >= self.frames.len() {
            if self.repeat == false {
                self.running = false;
                return None;
            } else {
                self.frame = 0;
            }
        }

        let frame = &self.frames[self.frame];
        self.timer = Timer::from_seconds(frame.length, TimerMode::Once);

        Some(frame)
    }
}

#[derive(Component)]
pub struct AnimationSet {
    set: HashMap<&'static str, Animation>,
}

impl AnimationSet {
    pub fn new(set: HashMap<&'static str, Animation>) -> Self {
        Self { set }
    }

    fn get(&self, key: &str) -> Option<&Animation> {
        self.set.get(key)
    }
}

fn animation_system(time: Res<Time>, mut query: Query<(&mut Animation, &mut TextureAtlasSprite)>) {
    for (mut animation, mut sprite) in &mut query {
        if !animation.running {
            continue;
        }

        animation.timer.tick(time.delta());

        if !animation.timer.just_finished() {
            continue;
        }

        if let Some(frame) = animation.next_frame() {
            sprite.index = frame.idx;
        }
    }
}
