use super::assets::Sprites;
use super::world::WorldPosition;
use super::GameState;
use bevy::prelude::*;

pub struct CannonPlugin;

impl bevy::app::Plugin for CannonPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(cannon_move_system.in_set(OnUpdate(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Cannon {
    pub track: usize,
    pub heading: Vec2,
    pub size: usize,
    forward: bool,
}

impl Cannon {
    pub fn new(size: usize, track: usize, heading: Vec2) -> Self {
        Self {
            track,
            heading,
            size,
            forward: true,
        }
    }

    pub fn spawn_offset(&self, note: i32) -> Vec2 {
        let offset = note as f32 * 16.;
        if self.heading.y.abs() > self.heading.x.abs() {
            if self.heading.y < 0. {
                Vec2::new(self.size as f32 * 16. - offset, 0.)
            } else {
                Vec2::new(offset, 0.)
            }
        } else {
            if self.heading.x < 0. {
                Vec2::new(0., self.size as f32 * 16. - offset)
            } else {
                Vec2::new(0., offset)
            }
        }
    }

    fn horizontal(&self) -> bool {
        self.heading.y.abs() > self.heading.x.abs()
    }
}

/// Spawn a single entity with `size` children as sprites for the cannons.
pub fn spawn_cannon(
    mut cannon: Cannon,
    commands: &mut Commands,
    position: Vec2,
    sprites: &Res<Sprites>,
) {
    cannon.heading = cannon.heading.normalize();
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

fn cannon_move_system(mut query: Query<(&mut Cannon, &mut WorldPosition)>) {
    for (mut cannon, mut cannon_pos) in &mut query {
        let delta = if cannon.forward { 1.0 } else { -1.0 };
        let movement = if cannon.horizontal() {
            Vec2::new(delta, 0.)
        } else {
            Vec2::new(0., delta)
        };

        cannon_pos.position += movement;
        if cannon.horizontal() {
            if cannon_pos.position.x < 32. {
                cannon.forward = true;
            } else if cannon_pos.position.x > (12. * 16.) {
                cannon.forward = false;
            }
        } else {
            if cannon_pos.position.y < 16. {
                cannon.forward = true;
            } else if cannon_pos.position.y > (5. * 16.) {
                cannon.forward = false;
            }
        }
    }
}
