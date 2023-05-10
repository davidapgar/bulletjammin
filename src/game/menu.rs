use super::audio::audio_generator::*;
use super::audio::Audio;
use super::song::{mary_song, techno, Song};
use super::{EndState, GameState};
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonColors>()
            .add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
            .add_system(click_play_button.in_set(OnUpdate(GameState::Menu)))
            .add_system(cleanup_menu.in_schedule(OnExit(GameState::Menu)))
            .add_system(game_over.in_schedule(OnEnter(GameState::GameOver)))
            .add_system(click_reset_button.in_set(OnUpdate(GameState::GameOver)))
            .add_system(cleanup_menu.in_schedule(OnExit(GameState::GameOver)));
    }
}

#[derive(Resource)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::rgb(0.15, 0.15, 0.15),
            hovered: Color::rgb(0.25, 0.25, 0.25),
        }
    }
}

#[derive(Component)]
struct MenuContainer;
#[derive(Component)]
enum WhichButton {
    Mary,
    Techno,
}

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_colors: Res<ButtonColors>,
) {
    let font = asset_server.load("fonts/NotJamSlabSerif1.ttf");
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    gap: Size::new(Val::Px(0.), Val::Px(16.0)),
                    ..default()
                },
                ..default()
            },
            MenuContainer,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Bullet Jammin'!",
                TextStyle {
                    font: font.clone(),
                    font_size: 60.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));

            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(240.0), Val::Px(50.0)),
                            margin: UiRect::all(Val::Auto),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: button_colors.normal.into(),
                        ..default()
                    },
                    WhichButton::Mary,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Mary's Jam",
                        TextStyle {
                            font: font.clone(),
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });

            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(240.0), Val::Px(50.0)),
                            margin: UiRect::all(Val::Auto),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: button_colors.normal.into(),
                        ..default()
                    },
                    WhichButton::Techno,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "What even is this?",
                        TextStyle {
                            font: font.clone(),
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });

            parent.spawn(TextBundle::from_section(
                "WASD or Arrows to move, left-click to shoot on the beat",
                TextStyle {
                    font: font.clone(),
                    font_size: 12.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });
}

fn click_play_button(
    mut commands: Commands,
    button_colors: Res<ButtonColors>,
    mut state: ResMut<NextState<GameState>>,
    mut interation_query: Query<
        (&Interaction, &mut BackgroundColor, &WhichButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, which) in &mut interation_query {
        match *interaction {
            Interaction::Clicked => {
                match which {
                    WhichButton::Mary => commands.insert_resource(mary_song()),
                    WhichButton::Techno => commands.insert_resource(techno()),
                }
                state.set(GameState::Playing);
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, button_query: Query<Entity, With<MenuContainer>>) {
    for button in &button_query {
        commands.entity(button).despawn_recursive();
    }
}

fn game_over(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_colors: Res<ButtonColors>,
    end_state: Res<EndState>,
    audio: Res<Audio>,
) {
    let font = asset_server.load("fonts/NotJamSlabSerif1.ttf");
    let game_over_text = match *end_state {
        EndState::GameOver => "GAME OVER!",
        EndState::Winner => "YOU WIN!",
    };
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            MenuContainer,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                game_over_text,
                TextStyle {
                    font: font.clone(),
                    font_size: 60.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));

            parent
                .spawn((ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(240.0), Val::Px(50.0)),
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: button_colors.normal.into(),
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Restart",
                        TextStyle {
                            font: font.clone(),
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        });

    match *end_state {
        EndState::GameOver => {}
        EndState::Winner => {
            let vco = Vco::from_oscillator(SuperSaw::new(1.), 130.81);
            let vca = Vca::new(vco, Envelope::new(0.2, 0.1, 0.2, 1.0));
            audio.play(vca.as_raw());
            let vco = Vco::from_oscillator(SuperSaw::new(1.), 196.00);
            let vca = Vca::new(vco, Envelope::new(0.2, 0.1, 0.2, 1.0));
            audio.play(vca.as_raw());
            let vco = Vco::from_oscillator(SuperSaw::new(1.), 261.63);
            let vca = Vca::new(vco, Envelope::new(0.2, 0.1, 0.2, 1.0));
            audio.play(vca.as_raw());
        }
    }
}

fn click_reset_button(
    mut commands: Commands,
    button_colors: Res<ButtonColors>,
    mut state: ResMut<NextState<GameState>>,
    mut interation_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interation_query {
        match *interaction {
            Interaction::Clicked => {
                commands.remove_resource::<Song>();
                commands.remove_resource::<EndState>();
                state.set(GameState::Menu);
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}
