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
pub struct Enemy {
    heading: Vec2,
    timer: Timer,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            heading: Vec2::default(),
            timer: Timer::new(bevy::utils::Duration::from_nanos(1), TimerMode::Once),
        }
    }
}

fn enemy_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut WorldPosition, &mut Enemy), (With<Enemy>, Without<Wall>)>,
    wall_query: Query<&WorldPosition, (With<Wall>, Without<Enemy>)>,
) {
    let mut rng = rand::thread_rng();
    let wall_size = Vec2::new(16., 16.);
    let enemy_size = Vec2::new(16., 12.);

    for (mut e_pos, mut enemy) in &mut query {
        enemy.timer.tick(time.delta());

        if enemy.timer.just_finished() {
            let movement = Vec2::new(rng.gen_range(-2.0..2.0), rng.gen_range(-2.0..2.0));
            enemy.heading = movement.normalize();

            enemy.timer = Timer::from_seconds(1.0, TimerMode::Once);
        }

        let movement = enemy.heading;
        e_pos.position += movement;

        for wall in &wall_query {
            if let Some(collision) = collide(
                e_pos.position.extend(0.),
                enemy_size,
                wall.position.extend(0.),
                wall_size,
            ) {
                match collision {
                    Collision::Left | Collision::Right => e_pos.position.x -= movement.x,
                    Collision::Bottom | Collision::Top => e_pos.position.y -= movement.y,
                    Collision::Inside => e_pos.position -= movement,
                }
            }
        }
    }
}
