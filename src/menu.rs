use bevy::{app::AppExit, prelude::*};

use crate::GameState;

#[derive(Default, Debug, States, Hash, Eq, PartialEq, Clone, Copy)]
enum MenuState {
    #[default]
    Main,
    Settings,
    Disabled, // not in menu, e.g. InGame state
}

// color constants
const TEXT_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);
const NORMAL_BUTTON: Color = Color::GRAY;
const HOVERED_BUTTON: Color = Color::DARK_GRAY;
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// tag entities added on the Main menu screen
#[derive(Component)]
struct OnMenuMainScreen;

// tag entities added on the Settings menu screen
#[derive(Component)]
struct OnMenuSettingsScreen;

#[derive(Component)]
struct SelectedOption;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Settings,
    BackToMainMenu,
    Quit,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .add_systems(OnEnter(GameState::Menu), menu_setup)
            .add_systems(OnEnter(MenuState::Main), main_menu_setup)
            .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMenuMainScreen>)
            .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
            .add_systems(
                OnExit(MenuState::Settings),
                despawn_screen::<OnMenuSettingsScreen>,
            )
            .add_systems(
                Update,
                (menu_action, button_colors).run_if(in_state(GameState::Menu)),
            );
    }
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match menu_button_action {
            MenuButtonAction::Play => {
                info!("You pressed Play button!");
                game_state.set(GameState::InGame);
                menu_state.set(MenuState::Disabled);
            }
            MenuButtonAction::Settings => {
                info!("You pressed Settings button!");
                menu_state.set(MenuState::Settings);
            }
            MenuButtonAction::Quit => {
                info!("You pressed Quit button!");
                app_exit_events.send(AppExit);
            }
            MenuButtonAction::BackToMainMenu => {
                info!("You pressed Back button!");
                menu_state.set(MenuState::Main);
            }
        }
    }
}

fn button_colors(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        };
    }
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut cmds: Commands) {
    info!("despaw_screen: called");
    for e in &to_despawn {
        cmds.entity(e).despawn_recursive();
    }
}

fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    info!("menu_setup: begin");
    menu_state.set(MenuState::Main); // This will queue up state transitions to be performed during the next frame update cycle.
}

fn main_menu_setup(mut cmds: Commands) {
    info!("main_menu_setup: begin");
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: TEXT_COLOR,
        ..default()
    };

    cmds.spawn((
        // pure color background
        // put everything in center
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::WHITE.into(),
            ..default()
        },
        OnMenuMainScreen,
    ))
    .with_children(|parent| {
        parent
            .spawn(NodeBundle {
                // put buttons in a column
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent.spawn(
                    // game name
                    TextBundle::from_section(
                        "geowartry",
                        TextStyle {
                            font_size: 80.0,
                            color: TEXT_COLOR,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    }),
                );

                // play button
                parent
                    .spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: Color::GRAY.into(),
                            ..default()
                        },
                        MenuButtonAction::Play,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "New Game",
                            button_text_style.clone(),
                        ));
                    });

                // settings button
                parent
                    .spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: Color::GRAY.into(),
                            ..default()
                        },
                        MenuButtonAction::Settings,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "Settings",
                            button_text_style.clone(),
                        ));
                    });

                // exit button
                parent
                    .spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: Color::GRAY.into(),
                            ..default()
                        },
                        MenuButtonAction::Quit,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section("Exit", button_text_style.clone()));
                    });
            });
    });
}

fn settings_menu_setup(mut cmds: Commands) {
    info!("settings_menu_setup: begin");
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: TEXT_COLOR,
        ..default()
    };

    cmds.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::WHITE.into(),
            ..default()
        },
        OnMenuSettingsScreen,
    ))
    .with_children(|parent| {
        parent
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        MenuButtonAction::BackToMainMenu,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section("Back", button_text_style.clone()));
                    });
            });
    });
}
