use bevy::prelude::*;
use bevy::utils::Duration;
use std::collections::HashMap;

pub struct AnimationPlugin;

impl bevy::app::Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        //app.add_system(animation_system);
    }
}

// WIP: figure out this API
pub trait AnimationMarker: PartialEq {
    fn animation(&self) -> Animation;
}

#[derive(Component)]
pub struct Animated<T: AnimationMarker> {
    stack: Vec<(T, Animation)>,
}

impl<T> Default for Animated<T>
where
    T: AnimationMarker,
{
    fn default() -> Self {
        Self { stack: vec![] }
    }
}

impl<T> Animated<T>
where
    T: AnimationMarker,
{
    pub fn push_animation(&mut self, animation_id: T) {
        // FIXME: Find existing same named animation.
        let animation = animation_id.animation();
        self.stack.push((animation_id, animation));
    }

    pub fn set_animation(&mut self, animation_id: T) {
        let mut found = false;
        for (id, _) in &self.stack {
            if animation_id == *id {
                found = true;
                break;
            }
        }
        if found {
            return;
        }

        self.stack.clear();
        let animation = animation_id.animation();
        self.stack.push((animation_id, animation));
    }

    pub fn tick(&mut self, delta: Duration) -> bool {
        if let Some((_, last)) = self.stack.last_mut() {
            last.timer.tick(delta);
            if last.timer.finished() {
                return true;
            }
        }
        return false;
    }

    pub fn next_frame(&mut self) -> Option<&AnimationFrame> {
        let mut found = false;
        while self.stack.len() != 0 {
            let (_, last) = self.stack.last_mut().unwrap();
            if let Some(_) = last.next_frame() {
                found = true;
                break;
            }

            self.stack.pop();
        }

        if found {
            let (_, last) = self.stack.last().unwrap();
            Some(last.current_frame())
        } else {
            None
        }
    }
}

pub struct AnimationFrame {
    pub idx: usize,
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

    fn current_frame(&self) -> &AnimationFrame {
        &self.frames[self.frame]
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

/*
 * TODO: If `Animated` takes a generic, then there must be a separate system for all animations
 * Or, if it's type erased, then interacting with the `Animated` component becomes (potentially)
 * clumsy - since the wrong "type" of AnimationMarker could be passed, or it has to be unerased
 * at every call sight.
 * For now, make a animation system per thingy that has an animation. It's jank, think of better
 * solutions.
fn animated_system(time: Res<Time>, mut query: Query<(&mut Animated, &mut TextureAtlasSprite)>) {
    for (mut animated, mut sprite) in &mut query {
        if animated.tick(time.delta()) {
            if let Some(frame) = animated.next_frame() {
                sprite.index = frame.idx;
            }
        }
    }
}
*/
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
