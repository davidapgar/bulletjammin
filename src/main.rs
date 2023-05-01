use bevy::prelude::*;
use bevy::DefaultPlugins;

mod game;

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bullet Jam".to_string(),
                        resolution: (800., 600.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        );

    app.add_plugin(game::Plugin);

    /*
    if cfg!(debug_assertions) {
        app.add_plugin(LogDiagnosticsPlugin::default());
    }
    */

    app.run();
}
