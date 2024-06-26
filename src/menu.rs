use bevy::{app::AppExit, prelude::*};
mod settings;
use super::GameState;
use settings::*;

#[derive(Default, Debug, States, Hash, Eq, PartialEq, Clone, Copy)]
enum MenuState {
    Main,
    Settings,
    #[default]
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

#[derive(Component, Clone, Copy)]
enum MenuButtonAction {
    Play,
    ToSettings,
    MusicUp,
    MusicDown,
    SoundUp,
    SoundDown,
    BackToMainMenu,
    QuitGame,
}

enum MenuItemType {
    Title,
    Button,
    SettingValue,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .insert_resource(GameSettings::default())
            .add_systems(OnEnter(GameState::Menu), setup)
            .add_systems(OnEnter(MenuState::Main), main_menu_setup)
            .add_systems(
                OnExit(MenuState::Main),
                despawn_screen::<OnMenuMainScreen>,
            )
            .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
            .add_systems(
                Update,
                refresh_setting_value.run_if(in_state(MenuState::Settings)),
            )
            .add_systems(
                OnExit(MenuState::Settings),
                despawn_screen::<OnMenuSettingsScreen>,
            )
            .add_systems(
                Update,
                (menu_action, refresh_button_colors)
                    .run_if(in_state(GameState::Menu)),
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
    mut game_settings: ResMut<GameSettings>,
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
            MenuButtonAction::ToSettings => {
                info!("You pressed Settings button!");
                menu_state.set(MenuState::Settings);
            }
            MenuButtonAction::QuitGame => {
                info!("You pressed Quit button!");
                app_exit_events.send(AppExit);
            }
            MenuButtonAction::BackToMainMenu => {
                info!("You pressed Back button!");
                menu_state.set(MenuState::Main);
            }
            MenuButtonAction::MusicUp => {
                if game_settings.music < GameSettings::MUSIC_MAX {
                    game_settings.music += 1;
                }
            }
            MenuButtonAction::MusicDown => {
                if game_settings.music > 0 {
                    game_settings.music -= 1;
                }
            }
            MenuButtonAction::SoundUp => {
                if game_settings.sound < GameSettings::SOUND_MAX {
                    game_settings.sound += 1;
                }
            }
            MenuButtonAction::SoundDown => {
                if game_settings.sound > 0 {
                    game_settings.sound -= 1;
                }
            }
        }
    }
}

fn refresh_button_colors(
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

fn refresh_setting_value(
    mut query: Query<(&mut Text, &GameSettingsType)>,
    game_settings: Res<GameSettings>,
) {
    let ts = TextStyle {
        font_size: 40.0,
        color: TEXT_COLOR,
        ..default()
    };
    for (mut text, &setting_type) in &mut query {
        let value = get_setting_value(&game_settings, setting_type);
        *text = Text::from_section(value.to_string(), ts.clone());
    }
}

fn despawn_screen<T: Component>(
    to_despawn: Query<Entity, With<T>>,
    mut cmds: Commands,
) {
    info!("despaw_screen: called");
    for e in &to_despawn {
        cmds.entity(e).despawn_recursive();
    }
}

fn setup(mut menu_state: ResMut<NextState<MenuState>>) {
    info!("menu_setup: begin");
    menu_state.set(MenuState::Main); // This will queue up state transitions to be performed during the next frame update cycle.
}

struct MenuItem<'a> {
    item_type: MenuItemType,
    text: &'a str,
    actions: Vec<MenuButtonAction>,
    value_type: Option<GameSettingsType>,
}

fn get_setting_value(
    game_settings: &Res<GameSettings>,
    value_type: GameSettingsType,
) -> usize {
    match value_type {
        GameSettingsType::Music => game_settings.music,
        GameSettingsType::Sound => game_settings.sound,
    }
}

fn setup_from_item_list(
    mut cmds: Commands,
    game_settings: Res<GameSettings>,
    comp: impl Component,
    item_list: Vec<MenuItem>, // item type, text, action(for buttons only, None for non-buttons)
) {
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let slim_button_style = Style {
        width: Val::Px(50.0),
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
    let slim_button_text_style = TextStyle {
        font_size: 20.0,
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
        comp, // root node is tagged, for the convenience of despawn()
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
                for i in item_list {
                    match i.item_type {
                        MenuItemType::Title => {
                            parent.spawn(
                                TextBundle::from_section(
                                    i.text,
                                    TextStyle {
                                        font_size: 80.0,
                                        color: TEXT_COLOR,
                                        ..default()
                                    },
                                )
                                .with_style(
                                    Style {
                                        margin: UiRect::all(Val::Px(50.0)),
                                        ..default()
                                    },
                                ),
                            );
                        }
                        MenuItemType::Button => {
                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: button_style.clone(),
                                        background_color: Color::GRAY.into(),
                                        ..default()
                                    },
                                    i.actions[0], // a button must have a button action
                                ))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        i.text,
                                        button_text_style.clone(),
                                    ));
                                });
                        }
                        MenuItemType::SettingValue => {
                            parent
                                .spawn(NodeBundle {
                                    // put value buttons in a row
                                    style: Style {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    // value name
                                    parent.spawn(TextBundle::from_section(
                                        i.text,
                                        button_text_style.clone(),
                                    ));
                                    // value down button
                                    parent
                                        .spawn((
                                            ButtonBundle {
                                                style: slim_button_style
                                                    .clone(),
                                                background_color: Color::GRAY
                                                    .into(),
                                                ..default()
                                            },
                                            i.actions[0],
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn(
                                                TextBundle::from_section(
                                                    "<",
                                                    slim_button_text_style
                                                        .clone(),
                                                ),
                                            );
                                        });
                                    // the value
                                    parent.spawn((
                                        TextBundle::from_section(
                                            get_setting_value(
                                                &game_settings,
                                                i.value_type.unwrap(),
                                            )
                                            .to_string(),
                                            button_text_style.clone(),
                                        ),
                                        i.value_type.unwrap(),
                                    ));
                                    // value up button
                                    parent
                                        .spawn((
                                            ButtonBundle {
                                                style: slim_button_style
                                                    .clone(),
                                                background_color: Color::GRAY
                                                    .into(),
                                                ..default()
                                            },
                                            i.actions[1],
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn(
                                                TextBundle::from_section(
                                                    ">",
                                                    slim_button_text_style
                                                        .clone(),
                                                ),
                                            );
                                        });
                                });
                        }
                    }
                }
            });
    });
}

fn main_menu_setup(cmds: Commands, game_settings: Res<GameSettings>) {
    info!("main_menu_setup: begin");

    // item type, text, action(button only, None for non-buttons)
    let item_list = vec![
        MenuItem {
            item_type: MenuItemType::Title,
            text: "geowartry",
            actions: vec![],
            value_type: None,
        },
        MenuItem {
            item_type: MenuItemType::Button,
            text: "Play",
            actions: vec![MenuButtonAction::Play],
            value_type: None,
        },
        MenuItem {
            item_type: MenuItemType::Button,
            text: "Settings",
            actions: vec![MenuButtonAction::ToSettings],
            value_type: None,
        },
        MenuItem {
            item_type: MenuItemType::Button,
            text: "Quit",
            actions: vec![MenuButtonAction::QuitGame],
            value_type: None,
        },
    ];
    setup_from_item_list(cmds, game_settings, OnMenuMainScreen, item_list);
}

fn settings_menu_setup(cmds: Commands, game_settings: Res<GameSettings>) {
    info!("settings_menu_setup: begin");

    let item_list = vec![
        MenuItem {
            item_type: MenuItemType::SettingValue,
            text: "Music",
            actions: vec![
                MenuButtonAction::MusicDown,
                MenuButtonAction::MusicUp,
            ],
            value_type: Some(GameSettingsType::Music),
        },
        MenuItem {
            item_type: MenuItemType::SettingValue,
            text: "Sound",
            actions: vec![
                MenuButtonAction::SoundDown,
                MenuButtonAction::SoundUp,
            ],
            value_type: Some(GameSettingsType::Sound),
        },
        MenuItem {
            item_type: MenuItemType::Button,
            text: "Back",
            actions: vec![MenuButtonAction::BackToMainMenu],
            value_type: None,
        },
    ];
    setup_from_item_list(cmds, game_settings, OnMenuSettingsScreen, item_list);
}
