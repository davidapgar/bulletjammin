use super::world::{Bullet, BulletType, Wall, WorldPosition};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};

// !!!!!!!!!
// ~Implement collisions
// health for player
// player stats on top of screen
// Tracker based sound generation, tied to bullet spawning
// Make it good
// Enemies? Or just run around?
// !!!!!!!!!

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl bevy::app::Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_input_system)
            .add_system(player_bullet_system);
    }
}

fn player_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut WorldPosition, (With<Player>, Without<Wall>)>,
    wall_query: Query<&WorldPosition, (With<Wall>, Without<Player>)>,
) {
    let mut player = query.single_mut();

    let mut movement = Vec2::default();
    if keyboard_input.pressed(KeyCode::Up) {
        movement.y += 2.;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        movement.y -= 2.;
    }
    if keyboard_input.pressed(KeyCode::Left) {
        movement.x -= 2.;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        movement.x += 2.;
    }

    player.position += movement;

    let tile_size = Vec2::new(16., 16.);
    for wall in &wall_query {
        if let Some(collision) = collide(
            player.position.extend(0.),
            tile_size,
            wall.position.extend(0.),
            tile_size,
        ) {
            // TODO FIXME: When colliding with top/bottom tiles, sometimes they return right/left (at
            // corners). This stops movement. Ideally do something better.
            match collision {
                Collision::Left | Collision::Right => player.position.x -= movement.x,
                Collision::Bottom | Collision::Top => player.position.y -= movement.y,
                Collision::Inside => player.position -= movement,
            }
        }
    }
}

fn player_bullet_system(
    mut commands: Commands,
    player_query: Query<&WorldPosition, (With<Player>, Without<Bullet>)>,
    bullet_query: Query<(Entity, &WorldPosition, &Bullet), (With<Bullet>, Without<Player>)>,
) {
    let bullet_size = Vec2::new(8., 8.);
    for player in &player_query {
        let player_pos = player.position.extend(0.);
        let player_size = Vec2::new(16., 16.);

        let filtered = bullet_query.iter().filter_map(|(entity, pos, bullet)| {
            if let BulletType::Enemy = bullet.0 {
                Some((entity, pos))
            } else {
                None
            }
        });
        for (entity, bullet) in filtered {
            if let Some(_) = collide(
                player_pos,
                player_size,
                bullet.position.extend(0.),
                bullet_size,
            ) {
                commands.entity(entity).despawn();
            }
        }
    }
}
