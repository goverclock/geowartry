use bevy::prelude::*;

mod game;
mod menu;
mod pause_menu;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    InGame,
    // Paused,  // TODO: implement PauseMenuPlugin
}

#[derive(Resource, Debug)]
struct GameSettings {
    music: usize, // volume of music, 0-8
    sound: usize, // volumn of sound, 0-8
}

#[derive(Component, Clone, Copy)]
enum GameSettingsType {
    Music,
    Sound,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl GameSettings {
    const DEFAULT: Self = Self { music: 2, sound: 2 };
    const MUSIC_MAX: usize = 8;
    const SOUND_MAX: usize = 8;
}

fn main() {
    App::new()
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_plugins((DefaultPlugins, game::GamePlugin, menu::MenuPlugin))
        .run();
}

fn setup(mut cmds: Commands) {
    cmds.insert_resource(GameSettings::default());
    cmds.spawn(Camera2dBundle::default());
}
