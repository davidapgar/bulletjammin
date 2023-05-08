use super::GameState;
use bevy::prelude::*;

pub struct AssetPlugin;

impl bevy::app::Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Sprites::default())
            .add_system(load_assets.in_schedule(OnEnter(GameState::Menu)));
    }
}

#[derive(Resource, Default)]
pub struct Sprites {
    pub player: Handle<TextureAtlas>,
    pub floor: Handle<TextureAtlas>,
    pub wall: Handle<TextureAtlas>,
    pub cannon: Handle<TextureAtlas>,
    pub sheep: Handle<TextureAtlas>,
    pub ram: Handle<TextureAtlas>,
    pub blast: Handle<TextureAtlas>,
    pub shot: Handle<TextureAtlas>,
}

fn load_assets(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut sprites: ResMut<Sprites>,
) {
    sprites.player = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/player.png"),
        Vec2::new(16.0, 16.0),
        9,
        1,
        None,
        None,
    ));
    sprites.floor = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/floor.png"),
        Vec2::new(16.0, 16.0),
        1,
        1,
        None,
        None,
    ));
    sprites.wall = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/wall.png"),
        Vec2::new(16.0, 16.0),
        1,
        1,
        None,
        None,
    ));
    sprites.sheep = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/sheep.png"),
        Vec2::new(16.0, 16.0),
        4,
        1,
        None,
        None,
    ));
    sprites.ram = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/ram.png"),
        Vec2::new(128.0, 128.0),
        4,
        1,
        None,
        None,
    ));
    sprites.cannon = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/cannon.png"),
        Vec2::new(16.0, 16.0),
        2,
        1,
        None,
        None,
    ));
    sprites.blast = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/blast.png"),
        Vec2::new(8.0, 8.0),
        2,
        1,
        None,
        None,
    ));
    sprites.shot = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/shot.png"),
        Vec2::new(8.0, 8.0),
        2,
        1,
        None,
        None,
    ));
}
