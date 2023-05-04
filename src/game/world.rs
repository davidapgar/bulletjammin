use super::audio::audio_generator::*;
use super::audio::Audio;
use super::player::Player;
use bevy::prelude::*;

pub struct WorldPlugin;

impl bevy::app::Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Sprites::default())
            .insert_resource(Song::default())
            .add_startup_system(world_startup)
            .add_system(transform_world_system)
            .add_system(spawn_system)
            .add_system(move_system);
    }
}

#[derive(Resource, Default)]
struct Sprites {
    blast: Handle<TextureAtlas>,
}

#[derive(Resource)]
struct Song {
    timer: Timer,
    idx: i32,
}

impl Default for Song {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            idx: 0,
        }
    }
}

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Bullet(BulletType);

enum BulletType {
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

    let (player_sprite, floor_sprite, blast_sprite) = (
        asset_server.load("sprites/player.png"),
        asset_server.load("sprites/floor.png"),
        asset_server.load("sprites/blast.png"),
    );
    let (player_atlas, floor_atlas, blast_atlas) = (
        TextureAtlas::from_grid(player_sprite, Vec2::new(16.0, 16.0), 8, 1, None, None),
        TextureAtlas::from_grid(floor_sprite, Vec2::new(16.0, 16.0), 1, 1, None, None),
        TextureAtlas::from_grid(blast_sprite, Vec2::new(8.0, 8.0), 2, 1, None, None),
    );

    let (player_handle, floor_handle, blast_handle) = (
        texture_atlases.add(player_atlas),
        texture_atlases.add(floor_atlas),
        texture_atlases.add(blast_atlas),
    );

    sprites.blast = blast_handle.clone();

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: player_handle,
            sprite: TextureAtlasSprite::new(0),
            ..default()
        },
        Player,
        WorldPosition::new(Vec2::new(0., 0.), 1.),
    ));

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: blast_handle,
            ..default()
        },
        Bullet(BulletType::Enemy),
        Moveable(Vec2::new(4., 0.)),
        WorldPosition::new(Vec2::new(0., 16.), 1.),
    ));

    spawn_world_grid(&mut commands, floor_handle);
}

fn spawn_world_grid(commands: &mut Commands, floor_handle: Handle<TextureAtlas>) {
    let (w, h) = (10, 10);

    commands.spawn(World {
        size: Vec2::new(w as f32, h as f32),
    });

    for x in 0..w {
        for y in 0..h {
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
    mut song: ResMut<Song>,
    time: Res<Time>,
    audio: Res<Audio>,
) {
    song.timer.tick(time.delta());

    if song.timer.just_finished() {
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: sprites.blast.clone(),
                ..default()
            },
            Bullet(BulletType::Enemy),
            Moveable(Vec2::new(4., 0.)),
            WorldPosition::new(Vec2::new(0., 16. * song.idx as f32), 1.),
        ));

        let note = match song.idx {
            0 => 0,
            1 => 2,
            2 => 4,
            3 => 5,
            4 => 7,
            5 => 5,
            6 => 4,
            7 => 2,
            8 => 0,
            _ => 0,
        };
        let frequency = frequency_per_volt(note as f32 / 120.0 + 0.2);
        let vca = Vca::new(
            Vcf::new(
                Vco::new(
                    SquareWave::new(frequency),
                    frequency,
                    Attenuator::new(RampWave::new(30.), 0.02),
                ),
                frequency,
                0.5,
            ),
            Envelope::new(0.3, 0.05, 0.05, 0.2),
        );
        audio.play(vca.as_raw());

        song.idx += 1;
        if song.idx >= 8 {
            song.idx = 0;
        }

        song.timer = Timer::from_seconds(0.25, TimerMode::Once);
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
    for (mut transform, world_position) in query.iter_mut() {
        *transform = Transform::from_translation(
            (world_position.position * 2.).extend(world_position.layer),
        )
        .with_scale(Vec3::splat(2.0));
    }
}
