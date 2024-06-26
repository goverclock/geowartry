use bevy::prelude::*;

mod diep_colors;
mod game;
mod layer;
mod menu;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    InGame,
    // Paused,  // TODO: implement PauseMenuPlugin
}

fn main() {
    App::new()
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_plugins((DefaultPlugins, game::GamePlugin, menu::MenuPlugin))
        .run();
}

fn setup(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle::default());
}
