use super::animation::{Animated, Animation, AnimationFrame};
use super::assets::Sprites;
use super::audio::Audio;
use super::cannon::{spawn_cannon, Cannon};
use super::enemy::Enemy;
use super::player::{OnBeat, Player, PlayerAnimations};
use super::song::{mary_song, Song};
use super::GameState;
use bevy::prelude::*;

pub struct WorldPlugin;

impl bevy::app::Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Sprites::default())
            .insert_resource(mary_song())
            .insert_resource(SongTimer::default())
            .add_system(world_startup.in_schedule(OnEnter(GameState::Playing)))
            .add_systems(
                (
                    spawn_system,
                    move_system,
                    transform_world_system.after(spawn_system),
                )
                    .in_set(OnUpdate(GameState::Playing)),
            );
    }
}

#[derive(Resource)]
struct SongTimer {
    timer: Timer,
    idx: usize,
}

impl Default for SongTimer {
    fn default() -> Self {
        SongTimer {
            timer: Timer::from_seconds(0.125, TimerMode::Once),
            idx: 0,
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

    let enemy_frames = vec![
        AnimationFrame::new(0, 0.250),
        AnimationFrame::new(1, 0.250),
        AnimationFrame::new(0, 0.250),
        AnimationFrame::new(2, 0.250),
    ];
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: sprites.sheep.clone(),
            sprite: TextureAtlasSprite::new(0),
            ..default()
        },
        Enemy::default(),
        WorldPosition::new(Vec2::new(8. * 16., 8. * 16.), 1.),
        Animation::new(enemy_frames, true),
    ));

    let boss_frames = vec![
        AnimationFrame::new(0, 0.250),
        AnimationFrame::new(1, 0.150),
        AnimationFrame::new(0, 0.250),
        AnimationFrame::new(2, 0.150),
    ];
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: sprites.ram.clone(),
            sprite: TextureAtlasSprite::new(0),
            ..default()
        },
        Enemy::default(),
        WorldPosition::new(Vec2::new(12. * 16., 12. * 16.), 1.),
        Animation::new(boss_frames, true),
    ));

    spawn_world_grid(&mut commands, sprites.floor.clone(), sprites.wall.clone());

    // Left
    spawn_cannon(
        Cannon::new(12, 0, Vec2::new(1., 0.)),
        &mut commands,
        Vec2::new(16., 16.),
        &sprites,
    );
    // Bottom
    spawn_cannon(
        Cannon::new(12, 1, Vec2::new(0., 1.)),
        &mut commands,
        Vec2::new(32., 16.),
        &sprites,
    );
    // Right
    spawn_cannon(
        Cannon::new(12, 2, Vec2::new(-1., 0.)),
        &mut commands,
        Vec2::new(24. * 16., 16.),
        &sprites,
    );
    // Top
    spawn_cannon(
        Cannon::new(12, 3, Vec2::new(0., -1.)),
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

fn spawn_system(
    mut commands: Commands,
    sprites: Res<Sprites>,
    mut song_timer: ResMut<SongTimer>,
    song: Res<Song>,
    time: Res<Time>,
    audio: Res<Audio>,
    mut on_beat: ResMut<OnBeat>,
    cannon_query: Query<(&Cannon, &WorldPosition)>,
) {
    // TODO: Handle multiple banks of cannons
    let (cannon, cannon_pos) = cannon_query.iter().find(|(c, _)| c.track == 0).unwrap();

    song_timer.timer.tick(time.delta());

    if song_timer.timer.finished() {
        on_beat.0 = !on_beat.0;

        for (idx, maybe_note) in song.note(song_timer.idx).into_iter().enumerate() {
            if let Some((note, source)) = maybe_note {
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
                        Animation::new(
                            vec![AnimationFrame::new(0, 0.250), AnimationFrame::new(1, 0.250)],
                            true,
                        ),
                    ));
                }
                audio.play(source);
            }
        }

        song_timer.idx += 1;
        if song_timer.idx >= song.len() {
            song_timer.idx = 0;
        }

        song_timer.timer = Timer::from_seconds(0.25, TimerMode::Once);
    }
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
