use super::animation::{Animated, Animation, AnimationFrame, AnimationMarker};
use super::audio::audio_generator::*;
use super::audio::Audio;
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
            (
                enemy_movement_system,
                enemy_bullet_system,
                enemy_animation_system,
            )
                .in_set(OnUpdate(GameState::Playing)),
        )
        .add_system(enemy_teardown.in_schedule(OnExit(GameState::GameOver)));
    }
}

pub struct EnemyKilledEvent(EnemyType);

#[derive(Copy, Clone)]
pub enum EnemyType {
    Basic,
    Boss,
}

#[derive(Component)]
pub struct Enemy {
    heading: Vec2,
    timer: Timer,
    health: i32,
    enemy_type: EnemyType,
}

impl Enemy {
    pub fn boss() -> Self {
        Self {
            heading: Vec2::default(),
            timer: Timer::default(),
            health: 32,
            enemy_type: EnemyType::Boss,
        }
    }
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            heading: Vec2::default(),
            timer: Timer::default(),
            health: 8,
            enemy_type: EnemyType::Basic,
        }
    }
}

/// Composite type for both types of enemies. Really should only use the right animations for each.
#[derive(PartialEq)]
pub enum EnemyAnimations {
    SheepWalk,
    SheepHurt,
    RamWalk,
    RamHurt,
}

impl AnimationMarker for EnemyAnimations {
    fn animation(&self) -> Animation {
        match self {
            EnemyAnimations::SheepWalk => Animation::new(
                vec![
                    AnimationFrame::new(0, 0.125),
                    AnimationFrame::new(1, 0.125),
                    AnimationFrame::new(0, 0.125),
                    AnimationFrame::new(2, 0.125),
                ],
                true,
            ),
            EnemyAnimations::SheepHurt => Animation::new(
                vec![
                    AnimationFrame::new(0, 0.0625),
                    AnimationFrame::new(3, 0.0625),
                    AnimationFrame::new(0, 0.0625),
                    AnimationFrame::new(3, 0.0625),
                    AnimationFrame::new(0, 0.0625),
                    AnimationFrame::new(3, 0.0625),
                ],
                false,
            ),
            EnemyAnimations::RamWalk => Animation::new(
                vec![AnimationFrame::new(0, 0.250), AnimationFrame::new(2, 0.125)],
                true,
            ),
            EnemyAnimations::RamHurt => Animation::new(
                vec![
                    AnimationFrame::new(0, 0.0625),
                    AnimationFrame::new(1, 0.0625),
                    AnimationFrame::new(2, 0.0625),
                    AnimationFrame::new(3, 0.0625),
                    AnimationFrame::new(0, 0.0625),
                    AnimationFrame::new(1, 0.0625),
                    AnimationFrame::new(2, 0.0625),
                    AnimationFrame::new(3, 0.0625),
                ],
                false,
            ),
        }
    }
}

fn enemy_movement_system(
    time: Res<Time>,
    mut query: Query<
        (
            &mut WorldPosition,
            &mut Enemy,
            &mut Animated<EnemyAnimations>,
        ),
        (With<Enemy>, Without<Wall>),
    >,
    wall_query: Query<&WorldPosition, (With<Wall>, Without<Enemy>)>,
) {
    let mut rng = rand::thread_rng();
    let wall_size = Vec2::new(16., 16.);
    let enemy_size = Vec2::new(16., 12.);

    for (mut e_pos, mut enemy, mut animated) in &mut query {
        enemy.timer.tick(time.delta());

        if enemy.timer.finished() {
            match enemy.enemy_type {
                EnemyType::Basic => animated.set_animation(EnemyAnimations::SheepWalk),
                EnemyType::Boss => animated.set_animation(EnemyAnimations::RamWalk),
            }

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
    mut enemy_query: Query<
        (
            Entity,
            &WorldPosition,
            &mut Enemy,
            &mut Animated<EnemyAnimations>,
        ),
        Without<Bullet>,
    >,
    bullet_query: Query<(Entity, &WorldPosition, &Bullet), Without<Player>>,
    mut event_writer: EventWriter<EnemyKilledEvent>,
    audio: Res<Audio>,
) {
    let bullet_size = Vec2::new(4., 4.);
    let enemy_size = Vec2::new(16., 12.);

    for (enemy_entity, enemy_position, mut enemy, mut animated) in &mut enemy_query {
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
                animated.push_animation(match enemy.enemy_type {
                    EnemyType::Basic => EnemyAnimations::SheepHurt,
                    EnemyType::Boss => EnemyAnimations::RamHurt,
                });
                enemy.health -= 1;
                commands.entity(entity).despawn();

                let vco = Vco::new(RampWave::new(440.), 440., RampWave::new(20.));
                let vca = Vca::new(vco, Envelope::new(0.1, 0.1, 0.0, 0.1));
                audio.play(vca.as_raw());
            }
        }
        if enemy.health <= 0 {
            commands.entity(enemy_entity).despawn();
            event_writer.send(EnemyKilledEvent(enemy.enemy_type));
        }
    }
}

fn enemy_animation_system(
    time: Res<Time>,
    mut enemy_query: Query<(&mut Animated<EnemyAnimations>, &mut TextureAtlasSprite)>,
) {
    for (mut animated, mut sprite) in &mut enemy_query {
        if animated.tick(time.delta()) {
            if let Some(frame) = animated.next_frame() {
                sprite.index = frame.idx;
            }
        }
    }
}

fn enemy_teardown(mut commands: Commands, query: Query<Entity, With<Enemy>>) {
    for enemy in &query {
        commands.entity(enemy).despawn();
    }
}
