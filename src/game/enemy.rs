use super::world::{Wall, WorldPosition};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
use rand::prelude::*;

pub struct EnemyPlugin;

impl bevy::app::Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(enemy_movement_system);
    }
}

#[derive(Component)]
pub struct Enemy;

fn enemy_movement_system(
    mut query: Query<&mut WorldPosition, (With<Enemy>, Without<Wall>)>,
    wall_query: Query<&WorldPosition, (With<Wall>, Without<Enemy>)>,
) {
    let mut rng = rand::thread_rng();
    let wall_size = Vec2::new(16., 16.);
    let enemy_size = Vec2::new(16., 12.);

    for mut enemy in &mut query {
        let movement = Vec2::new(rng.gen_range(-2.0..2.0), rng.gen_range(-2.0..2.0));

        enemy.position += movement;

        for wall in &wall_query {
            if let Some(collision) = collide(
                enemy.position.extend(0.),
                enemy_size,
                wall.position.extend(0.),
                wall_size,
            ) {
                match collision {
                    Collision::Left | Collision::Right => enemy.position.x -= movement.x,
                    Collision::Bottom | Collision::Top => enemy.position.y -= movement.y,
                    Collision::Inside => enemy.position -= movement,
                }
            }
        }
    }
}
