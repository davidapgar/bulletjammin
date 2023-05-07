use super::assets::Sprites;
use super::world::WorldPosition;
use super::GameState;
use bevy::prelude::*;

// TODO: Needed?
pub struct CannonPlugin;

#[derive(Component)]
pub struct Cannon {
    track: usize,
    heading: Vec2,
    size: usize,
}

impl Cannon {
    pub fn new(size: usize, track: usize, heading: Vec2) -> Self {
        Self {
            track,
            heading,
            size,
        }
    }
}

/// Spawn a single entity with `size` children as sprites for the cannons
/// TODO: Handle different headings
pub fn spawn_cannon(
    cannon: Cannon,
    commands: &mut Commands,
    position: Vec2,
    sprites: &Res<Sprites>,
) {
    let n_cannon = cannon.size;
    commands
        .spawn((
            cannon,
            WorldPosition::new(position, 1.0),
            SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., -1.)),
                ..default()
            },
        ))
        .with_children(|parent| {
            for y in 0..n_cannon {
                parent.spawn(SpriteSheetBundle {
                    texture_atlas: sprites.cannon.clone(),
                    transform: Transform::from_translation(Vec3::new(0., y as f32 * 16., 0.)),
                    ..default()
                });
            }
        });
}
