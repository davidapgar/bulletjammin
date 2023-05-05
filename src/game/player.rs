use super::world::{Bullet, BulletType, Wall, WorldPosition};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};

// !!!!!!!!!
// ~Implement collisions
// ~health for player
// ~player stats on top of screen
// ~Tracker based sound generation, tied to bullet spawning
//  partial, it's a bit tightly coupled.
// Make it good
// Enemies? Or just run around?
// Animations
// !!!!!!!!!

#[derive(Component)]
pub struct Player {
    health: i32,
}

impl Default for Player {
    fn default() -> Self {
        Player { health: 8 }
    }
}

pub struct PlayerPlugin;

impl bevy::app::Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(health_ui_startup_system)
            .add_system(player_input_system)
            .add_system(player_bullet_system)
            .add_system(update_health_system);
    }
}

#[derive(Component)]
struct HeartUi(bool, i32);

fn health_ui_startup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let heart_sprite = asset_server.load("sprites/heart.png");
    let heart_atlas = TextureAtlas::from_grid(heart_sprite, Vec2::new(8., 8.), 2, 1, None, None);
    let heart_handle = texture_atlases.add(heart_atlas);

    for i in 0..8 {
        let translation = Vec3::new(-400. + 8. + (i as f32 * 16.), 300. - 8., 900.);
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: heart_handle.clone(),
                transform: Transform::from_translation(translation).with_scale(Vec3::splat(2.0)),
                ..default()
            },
            HeartUi(true, i),
        ));
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
    mut player_query: Query<(&WorldPosition, &mut Player), Without<Bullet>>,
    bullet_query: Query<(Entity, &WorldPosition, &Bullet), (With<Bullet>, Without<Player>)>,
) {
    let bullet_size = Vec2::new(8., 8.);
    for (player_position, mut player) in &mut player_query {
        let player_pos = player_position.position.extend(0.);
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
                player.health -= 1;
                commands.entity(entity).despawn();
            }
        }
    }
}

fn update_health_system(
    player_query: Query<&Player>,
    mut health_query: Query<(&HeartUi, &mut TextureAtlasSprite)>,
) {
    let player = player_query.single();

    for (heart_ui, mut sprite) in &mut health_query {
        if heart_ui.1 >= player.health {
            sprite.index = 1;
        } else {
            sprite.index = 0;
        }
    }
}