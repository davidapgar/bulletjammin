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

/// Spawn a single entity with `size` children as sprites for the cannons.
pub fn spawn_cannon(
    cannon: Cannon,
    commands: &mut Commands,
    position: Vec2,
    sprites: &Res<Sprites>,
) {
    let n_cannon = cannon.size;

    let (sprite_index, flip_x, flip_y, vert) = {
        if cannon.heading.y.abs() > cannon.heading.x.abs() {
            if cannon.heading.y < 0. {
                (1, false, true, false)
            } else {
                (1, false, false, false)
            }
        } else {
            if cannon.heading.x < 0. {
                (0, true, false, true)
            } else {
                (0, false, false, true)
            }
        }
    };

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
            for idx in 0..n_cannon {
                let (x, y) = if vert {
                    (0., idx as f32 * 16.)
                } else {
                    (idx as f32 * 16., 0.)
                };

                parent.spawn(SpriteSheetBundle {
                    texture_atlas: sprites.cannon.clone(),
                    sprite: TextureAtlasSprite {
                        index: sprite_index,
                        flip_x,
                        flip_y,
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(x, y, 0.)),
                    ..default()
                });
            }
        });
}
