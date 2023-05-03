use bevy::prelude::*;

pub struct WorldPlugin;

impl bevy::app::Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(world_startup);
    }
}

fn world_startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());

    let (player_sprite, floor_sprite) = (
        asset_server.load("sprites/player.png"),
        asset_server.load("sprites/floor2.png"),
    );
    let (player_atlas, floor_atlas) = (
        TextureAtlas::from_grid(player_sprite, Vec2::new(16.0, 16.0), 8, 1, None, None),
        TextureAtlas::from_grid(floor_sprite, Vec2::new(16.0, 16.0), 1, 1, None, None),
    );

    let (player_handle, floor_handle) = (
        texture_atlases.add(player_atlas),
        texture_atlases.add(floor_atlas),
    );

    commands.spawn((SpriteSheetBundle {
        texture_atlas: player_handle,
        sprite: TextureAtlasSprite::new(0),
        transform: Transform::from_translation(Vec3::new(0., 0., 1.)).with_scale(Vec3::splat(2.0)),
        ..default()
    },));

    commands.spawn((SpriteSheetBundle {
        texture_atlas: floor_handle,
        sprite: TextureAtlasSprite::new(0),
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)).with_scale(Vec3::splat(2.0)),
        ..default()
    },));
}
