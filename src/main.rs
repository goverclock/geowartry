use bevy::prelude::*;

mod game;
mod menu;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Game,
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
