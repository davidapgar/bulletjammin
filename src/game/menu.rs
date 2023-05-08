use super::GameState;
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonColors>()
            .add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
            .add_system(click_play_button.in_set(OnUpdate(GameState::Menu)))
            .add_system(cleanup_menu.in_schedule(OnExit(GameState::Menu)))
            .add_system(game_over.in_schedule(OnEnter(GameState::GameOver)));
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
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(120.0), Val::Px(50.0)),
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: button_colors.normal.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font: font.clone(),
                            font_size: 40.0,
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

fn game_over(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/NotJamSlabSerif1.ttf");
    commands
        .spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "GAME OVER!",
                TextStyle {
                    font: font.clone(),
                    font_size: 60.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });
}
