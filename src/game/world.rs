use super::animation::{Animation, AnimationFrame};
use super::audio::audio_generator::*;
use super::audio::Audio;
use super::player::Player;
use super::song::{mary_song, Song};
use bevy::prelude::*;

pub struct WorldPlugin;

impl bevy::app::Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Sprites::default())
            .insert_resource(mary_song())
            .insert_resource(SongTimer::default())
            .add_startup_system(world_startup)
            .add_system(spawn_system)
            .add_system(move_system)
            .add_system(transform_world_system.after(spawn_system));
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

#[derive(Resource, Default)]
struct Sprites {
    blast: Handle<TextureAtlas>,
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
struct World {
    size: Vec2,
}

impl World {
    fn in_bounds(&self, pos: &Vec2) -> bool {
        pos.x >= 0. && pos.y >= 0. && pos.x <= self.size.x * 16. && pos.y <= self.size.y * 16.
    }
}

#[derive(Component)]
struct Moveable(Vec2);

impl WorldPosition {
    pub fn new(position: Vec2, layer: f32) -> Self {
        Self { position, layer }
    }
}

fn world_startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut sprites: ResMut<Sprites>,
) {
    commands.spawn(Camera2dBundle::default());

    let (player_sprite, floor_sprite, wall_sprite, blast_sprite) = (
        asset_server.load("sprites/player.png"),
        asset_server.load("sprites/floor.png"),
        asset_server.load("sprites/wall.png"),
        asset_server.load("sprites/blast.png"),
    );

    let (player_atlas, floor_atlas, wall_atlas, blast_atlas) = (
        TextureAtlas::from_grid(player_sprite, Vec2::new(16.0, 16.0), 8, 1, None, None),
        TextureAtlas::from_grid(floor_sprite, Vec2::new(16.0, 16.0), 1, 1, None, None),
        TextureAtlas::from_grid(wall_sprite, Vec2::new(16.0, 16.0), 1, 1, None, None),
        TextureAtlas::from_grid(blast_sprite, Vec2::new(8.0, 8.0), 2, 1, None, None),
    );

    let (player_handle, floor_handle, wall_handle, blast_handle) = (
        texture_atlases.add(player_atlas),
        texture_atlases.add(floor_atlas),
        texture_atlases.add(wall_atlas),
        texture_atlases.add(blast_atlas),
    );

    sprites.blast = blast_handle.clone();

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: player_handle,
            sprite: TextureAtlasSprite::new(0),
            ..default()
        },
        Player::default(),
        WorldPosition::new(Vec2::new(5. * 16., 5. * 16.), 1.),
        Animation::new(
            vec![AnimationFrame::new(0, 0.125), AnimationFrame::new(1, 0.025)],
            true,
        ),
    ));

    spawn_world_grid(&mut commands, floor_handle, wall_handle);
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
) {
    song_timer.timer.tick(time.delta());

    if song_timer.timer.just_finished() {
        if let Some((note, source)) = song.note(song_timer.idx) {
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: sprites.blast.clone(),
                    transform: Transform::from_translation(Vec3::new(0., 0., -1.)),
                    ..default()
                },
                Bullet(BulletType::Enemy),
                Moveable(Vec2::new(4., 0.)),
                WorldPosition::new(Vec2::new(0., 16. * note as f32), 1.),
            ));
            audio.play(source);
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
