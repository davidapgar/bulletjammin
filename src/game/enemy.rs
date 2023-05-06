use super::player::Player;
use super::world::{Bullet, BulletType, Wall, WorldPosition};
use super::GameState;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
use rand::prelude::*;

pub struct EnemyPlugin;

impl bevy::app::Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (enemy_movement_system, enemy_bullet_system).in_set(OnUpdate(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Enemy {
    heading: Vec2,
    timer: Timer,
    health: i32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            heading: Vec2::default(),
            timer: Timer::new(bevy::utils::Duration::from_nanos(1), TimerMode::Once),
            health: 8,
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

fn enemy_bullet_system(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &WorldPosition, &mut Enemy), Without<Bullet>>,
    bullet_query: Query<(Entity, &WorldPosition, &Bullet), Without<Player>>,
) {
    let bullet_size = Vec2::new(4., 4.);
    let enemy_size = Vec2::new(16., 12.);

    for (enemy_entity, enemy_position, mut enemy) in &mut enemy_query {
        let enemy_pos = enemy_position.position.extend(0.);

        let filtered = bullet_query
            .iter()
            .filter_map(|(entity, pos, bullet)| match bullet.0 {
                BulletType::Player => Some((entity, pos)),
                _ => None,
            });
        for (entity, bullet_pos) in filtered {
            if let Some(_) = collide(
                enemy_pos,
                enemy_size,
                bullet_pos.position.extend(0.),
                bullet_size,
            ) {
                enemy.health -= 1;
                commands.entity(entity).despawn();
            }
        }
        if enemy.health <= 0 {
            commands.entity(enemy_entity).despawn();
        }
    }
}
