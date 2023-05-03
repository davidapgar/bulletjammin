use super::world::WorldPosition;
use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl bevy::app::Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_input_system);
    }
}

fn player_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut WorldPosition, With<Player>>,
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
}
