use super::animation::{Animated, Animation, AnimationFrame, AnimationMarker};
use super::assets::Sprites;
use super::audio::{audio_generator::*, Audio};
use super::world::{Bullet, BulletType, Moveable, Wall, WorldPosition};
use super::GameState;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};

// !!!!!!!!!
// ~Implement collisions
// ~health for player
// ~Implement game over
// ~player stats on top of screen
// ~Tracker based sound generation, tied to bullet spawning
//  Moar tracks, tie correctly to cannon clusters.
// ~Progress the song based on <something>
// ~Only shoot on the "beat"
// ~Make this tied to track 0 triggering (drum track)
// Make it good
// ~Enemies
//  Add spawning, removing progresses the song.
// ~Animations
// ~partial. Need stacking animations, repeatable.
// ~Apply to everything...
// !!!!!!!!!

pub struct PlayerPlugin;

#[derive(Resource, Default)]
pub struct OnBeat(pub bool);

impl bevy::app::Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OnBeat>()
            .add_system(health_ui_startup_system.in_schedule(OnEnter(GameState::Playing)))
            .add_systems(
                (
                    player_input_system,
                    player_bullet_system,
                    player_shooting_system,
                    player_animation_system,
                    update_health_system,
                )
                    .in_set(OnUpdate(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct Player {
    health: i32,
    cooldown: Timer,
    facing: Facing,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            health: 8,
            cooldown: Timer::default(),
            facing: Facing::Down,
        }
    }
}

enum Facing {
    Down,
    Up,
    Right,
    Left,
}

impl Facing {
    fn from(heading: Vec2) -> Self {
        if heading.x.abs() > heading.y.abs() {
            if heading.x > 0. {
                Facing::Right
            } else {
                Facing::Left
            }
        } else {
            if heading.y > 0. {
                Facing::Up
            } else {
                Facing::Down
            }
        }
    }
}

#[derive(PartialEq)]
pub enum PlayerAnimations {
    Down,
    DownShoot,
    Up,
    UpShoot,
    Left,
    LeftShoot,
    Right,
    RightShoot,
    Hurt,
}

impl AnimationMarker for PlayerAnimations {
    fn animation(&self) -> Animation {
        match self {
            PlayerAnimations::Down => Animation::new(vec![AnimationFrame::new(0, 0.1)], true),
            PlayerAnimations::DownShoot => Animation::new(
                vec![AnimationFrame::new(0, 0.1), AnimationFrame::new(1, 0.1)],
                true,
            ),
            PlayerAnimations::Up => Animation::new(vec![AnimationFrame::new(2, 0.1)], true),
            PlayerAnimations::UpShoot => Animation::new(
                vec![AnimationFrame::new(2, 0.1), AnimationFrame::new(3, 0.1)],
                true,
            ),
            PlayerAnimations::Right => Animation::new(vec![AnimationFrame::new(4, 0.1)], true),
            PlayerAnimations::RightShoot => Animation::new(
                vec![AnimationFrame::new(4, 0.1), AnimationFrame::new(5, 0.1)],
                true,
            ),
            PlayerAnimations::Left => Animation::new(vec![AnimationFrame::new(6, 0.1)], true),
            PlayerAnimations::LeftShoot => Animation::new(
                vec![AnimationFrame::new(6, 0.1), AnimationFrame::new(7, 0.1)],
                true,
            ),
            PlayerAnimations::Hurt => Animation::new(
                vec![
                    AnimationFrame::new(0, 0.1),
                    AnimationFrame::new(8, 0.1),
                    AnimationFrame::new(0, 0.1),
                    AnimationFrame::new(8, 0.1),
                    AnimationFrame::new(0, 0.1),
                    AnimationFrame::new(8, 0.1),
                ],
                false,
            ),
        }
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
    if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
        movement.y += 1.;
    }
    if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
        movement.y -= 1.;
    }
    if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
        movement.x -= 1.;
    }
    if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
        movement.x += 1.;
    }
    movement = movement.normalize_or_zero() * 2.;

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

fn player_shooting_system(
    mut commands: Commands,
    time: Res<Time>,
    sprites: Res<Sprites>,
    on_beat: Res<OnBeat>,
    windows: Query<&Window>,
    mut player_query: Query<(&WorldPosition, &mut Player, &mut Animated<PlayerAnimations>)>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    let window = windows.single();
    let (p_pos, mut player, mut animated) = player_query.single_mut();
    player.cooldown.tick(time.delta());

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let heading = (cursor_position - p_pos.position * 2.).normalize_or_zero();
    player.facing = Facing::from(heading);

    if mouse_button_input.pressed(MouseButton::Left) {
        if player.cooldown.finished() && on_beat.0 {
            let animation = match player.facing {
                Facing::Down => PlayerAnimations::DownShoot,
                Facing::Up => PlayerAnimations::UpShoot,
                Facing::Right => PlayerAnimations::RightShoot,
                Facing::Left => PlayerAnimations::LeftShoot,
            };
            animated.set_animation(animation);

            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: sprites.shot.clone(),
                    transform: Transform::from_translation(Vec3::new(0., 0., -1.)),
                    ..default()
                },
                Bullet(BulletType::Player),
                Moveable(heading * 4.0),
                WorldPosition::new(p_pos.position, 1.),
            ));
            player.cooldown = Timer::from_seconds(0.1, TimerMode::Once);
        }
    } else {
        let animation = match player.facing {
            Facing::Down => PlayerAnimations::Down,
            Facing::Up => PlayerAnimations::Up,
            Facing::Right => PlayerAnimations::Right,
            Facing::Left => PlayerAnimations::Left,
        };
        animated.set_animation(animation);
    }
}

fn player_bullet_system(
    mut commands: Commands,
    mut player_query: Query<
        (&WorldPosition, &mut Player, &mut Animated<PlayerAnimations>),
        Without<Bullet>,
    >,
    bullet_query: Query<(Entity, &WorldPosition, &Bullet), (With<Bullet>, Without<Player>)>,
    mut state: ResMut<NextState<GameState>>,
    audio: Res<Audio>,
) {
    let bullet_size = Vec2::new(8., 8.);
    for (player_position, mut player, mut animated) in &mut player_query {
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
                animated.push_animation(PlayerAnimations::Hurt);
                commands.entity(entity).despawn();

                let vco = Vco::new(RampWave::new(440.), 440., SawWave::new(20.));
                let vca = Vca::new(vco, Envelope::new(0.2, 0.1, 0.0, 0.1));
                audio.play(vca.as_raw());

                if player.health <= 0 {
                    state.set(GameState::GameOver);
                }
            }
        }
    }
}

fn player_animation_system(
    time: Res<Time>,
    mut player_query: Query<(
        &Player,
        &mut Animated<PlayerAnimations>,
        &mut TextureAtlasSprite,
    )>,
) {
    for (_player, mut animated, mut sprite) in &mut player_query {
        if animated.tick(time.delta()) {
            if let Some(frame) = animated.next_frame() {
                sprite.index = frame.idx;
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
