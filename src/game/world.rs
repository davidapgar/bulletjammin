use super::animation::{Animated, AnimationFrame};
use super::assets::Sprites;
use super::audio::Audio;
use super::cannon::{spawn_cannon, Cannon};
use super::enemy::{Enemy, EnemyAnimations, EnemyKilledEvent, EnemyType};
use super::player::{OnBeat, Player, PlayerAnimations};
use super::song::{mary_song, Song};
use super::GameState;
use bevy::prelude::*;
use rand::prelude::*;

pub struct WorldPlugin;

impl bevy::app::Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Sprites::default())
            .insert_resource(mary_song())
            .insert_resource(SongTimer::default())
            .add_event::<EnemyKilledEvent>()
            .add_system(world_startup.in_schedule(OnEnter(GameState::Playing)))
            .add_systems(
                (
                    spawn_system,
                    move_system,
                    enemy_spawn_system,
                    song_progression_system,
                    transform_world_system.after(spawn_system),
                )
                    .in_set(OnUpdate(GameState::Playing)),
            );
    }
}

// 120 BPM, 60 seconds/min, 4/beat (16th notes)
const BPM_TIMER_TIME: f32 = 0.125;

#[derive(Resource)]
struct SongTimer {
    timer: Timer,
    idx: usize,
    chain: usize,
    next_chain: bool,
}

impl Default for SongTimer {
    fn default() -> Self {
        SongTimer {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            idx: 0,
            chain: 0,
            next_chain: false,
        }
    }
}

#[derive(Component)]
struct Background;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Bullet(pub BulletType);

pub enum BulletType {
    Player,
    Enemy,
}

#[derive(Component)]
pub struct WorldPosition {
    pub position: Vec2,
    pub layer: f32,
}

// TODO: This should likely be a resource. Also does this make sense?
// It could/should hold children?
#[derive(Component)]
pub struct World {
    pub size: Vec2,
    next_spawn: Timer,
    enemy_spawned: usize,
    active_enemy: usize,
    enemy_killed: usize,
}

impl World {
    fn in_bounds(&self, pos: &Vec2) -> bool {
        pos.x >= 0. && pos.y >= 0. && pos.x <= self.size.x * 16. && pos.y <= self.size.y * 16.
    }
}

#[derive(Component)]
pub struct Moveable(pub Vec2);

impl WorldPosition {
    pub fn new(position: Vec2, layer: f32) -> Self {
        Self { position, layer }
    }
}

fn world_startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut sprites: Res<Sprites>,
) {
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: sprites.player.clone(),
            sprite: TextureAtlasSprite::new(0),
            ..default()
        },
        Player::default(),
        WorldPosition::new(Vec2::new(5. * 16., 5. * 16.), 1.),
        Animated::<PlayerAnimations>::default(),
    ));

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: sprites.sheep.clone(),
            sprite: TextureAtlasSprite::new(0),
            ..default()
        },
        Enemy::default(),
        WorldPosition::new(Vec2::new(8. * 16., 8. * 16.), 1.),
        Animated::<EnemyAnimations>::default(),
    ));

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: sprites.ram.clone(),
            sprite: TextureAtlasSprite::new(0),
            ..default()
        },
        Enemy::boss(),
        WorldPosition::new(Vec2::new(12. * 16., 12. * 16.), 1.),
        Animated::<EnemyAnimations>::default(),
    ));

    spawn_world_grid(&mut commands, sprites.floor.clone(), sprites.wall.clone());

    // Left
    spawn_cannon(
        Cannon::new(12, 1, Vec2::new(1., 0.)),
        &mut commands,
        Vec2::new(16., 16.),
        &sprites,
    );
    // Bottom
    spawn_cannon(
        Cannon::new(12, 0, Vec2::new(0., 1.)),
        &mut commands,
        Vec2::new(32., 16.),
        &sprites,
    );
    // Right
    spawn_cannon(
        Cannon::new(12, 1, Vec2::new(-1., 0.)),
        &mut commands,
        Vec2::new(24. * 16., 16.),
        &sprites,
    );
    // Top
    spawn_cannon(
        Cannon::new(12, 0, Vec2::new(0., -1.)),
        &mut commands,
        Vec2::new((24. - 12.) * 16., 16. * 16.),
        &sprites,
    );
}

fn spawn_world_grid(
    commands: &mut Commands,
    floor_handle: Handle<TextureAtlas>,
    wall_handle: Handle<TextureAtlas>,
) {
    let (w, h) = (25, 17);

    commands.spawn(World {
        size: Vec2::new(w as f32, h as f32),
        next_spawn: Timer::from_seconds(1.0, TimerMode::Once),
        enemy_spawned: 2,
        active_enemy: 2,
        enemy_killed: 0,
    });

    for x in [0, w] {
        for y in 0..h + 1 {
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: wall_handle.clone(),
                    ..default()
                },
                Wall,
                WorldPosition::new(Vec2::new((x * 16) as f32, (y * 16) as f32), 0.),
            ));
        }
    }
    for y in [0, h] {
        for x in 1..w {
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: wall_handle.clone(),
                    ..default()
                },
                Wall,
                WorldPosition::new(Vec2::new((x * 16) as f32, (y * 16) as f32), 0.),
            ));
        }
    }
    for x in 1..w {
        for y in 1..h {
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: floor_handle.clone(),
                    sprite: TextureAtlasSprite::new(0),
                    ..default()
                },
                Background,
                WorldPosition::new(Vec2::new((x * 16) as f32, (y * 16) as f32), 0.),
            ));
        }
    }
}

fn song_progression_system(
    mut song_timer: ResMut<SongTimer>,
    mut event_reader: EventReader<EnemyKilledEvent>,
    mut world_query: Query<&mut World>,
) {
    let mut world = world_query.single_mut();

    for _enemy_type in event_reader.iter() {
        world.active_enemy -= 1;
        world.enemy_killed += 1;

        let killed = world.enemy_killed;
        if killed % 2 == 0 {
            song_timer.next_chain = true;
        }
    }
}

fn spawn_system(
    mut commands: Commands,
    sprites: Res<Sprites>,
    mut song_timer: ResMut<SongTimer>,
    song: Res<Song>,
    time: Res<Time>,
    audio: Res<Audio>,
    mut state: ResMut<NextState<GameState>>,
    mut on_beat: ResMut<OnBeat>,
    cannon_query: Query<(&Cannon, &WorldPosition)>,
) {
    song_timer.timer.tick(time.delta());

    if song_timer.timer.finished() {
        on_beat.0 = false;

        for (idx, maybe_note) in song
            .note(song_timer.idx, song_timer.chain)
            .into_iter()
            .enumerate()
        {
            if let Some((note, source)) = maybe_note {
                if idx == 0 {
                    on_beat.0 = true;
                }
                for (cannon, cannon_pos) in cannon_query.iter().filter(|(c, _)| c.track == idx) {
                    let (spawn_pos, heading) = (
                        cannon_pos.position + cannon.spawn_offset(note),
                        cannon.heading * 4.,
                    );

                    commands.spawn((
                        SpriteSheetBundle {
                            texture_atlas: sprites.blast.clone(),
                            transform: Transform::from_translation(Vec3::new(0., 0., -1.)),
                            ..default()
                        },
                        Bullet(BulletType::Enemy),
                        Moveable(heading),
                        WorldPosition::new(spawn_pos, 1.),
                    ));
                }
                audio.play(source);
            }
        }

        song_timer.idx += 1;
        if song_timer.idx >= song.len(song_timer.chain) {
            song_timer.idx = 0;
            if song_timer.next_chain {
                song_timer.chain += 1;
                song_timer.next_chain = false;

                if song_timer.chain >= song.max_chains() {
                    state.set(GameState::Winner);
                }
            }
        }

        song_timer.timer = Timer::from_seconds(BPM_TIMER_TIME, TimerMode::Once);
    }
}

fn enemy_spawn_system(
    mut commands: Commands,
    sprites: Res<Sprites>,
    time: Res<Time>,
    player_query: Query<&WorldPosition, With<Player>>,
    mut world_query: Query<&mut World>,
) {
    let mut world = world_query.single_mut();

    world.next_spawn.tick(time.delta());
    if !world.next_spawn.finished() {
        return;
    }

    world.next_spawn = Timer::from_seconds(1.0, TimerMode::Once);

    if world.active_enemy > 5 {
        return;
    }

    world.enemy_spawned += 1;
    world.active_enemy += 1;

    let player_pos = player_query.single();
    let mut rng = rand::thread_rng();

    let player_cell = player_pos.position / 16.;
    let mut enemy_cell = Vec2::new(rng.gen_range(2..24) as f32, rng.gen_range(2..16) as f32);
    while player_cell == enemy_cell {
        enemy_cell = Vec2::new(rng.gen_range(2..24) as f32, rng.gen_range(2..16) as f32);
    }

    enemy_cell *= 16.;

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: sprites.sheep.clone(),
            sprite: TextureAtlasSprite::new(0),
            ..default()
        },
        Enemy::default(),
        WorldPosition::new(enemy_cell, 1.),
        Animated::<EnemyAnimations>::default(),
    ));
}

fn move_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WorldPosition, &Moveable)>,
    world_query: Query<&World>,
) {
    let world = world_query.single();

    for (entity, mut world_position, moveable) in query.iter_mut() {
        world_position.position += moveable.0;
        if !world.in_bounds(&world_position.position) {
            commands.entity(entity).despawn();
        }
    }
}

fn transform_world_system(mut query: Query<(&mut Transform, &WorldPosition)>) {
    let world_offset = Vec2::new(25. * 8., 18. * 8.);
    for (mut transform, world_position) in query.iter_mut() {
        *transform = Transform::from_translation(
            ((world_position.position - world_offset) * 2.).extend(world_position.layer),
        )
        .with_scale(Vec3::splat(2.0));
    }
}
