use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{GameSettings, GameState};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Game>()
            .add_systems(OnEnter(GameState::InGame), setup)
            .add_systems(OnExit(GameState::InGame), cleanup)
            .add_systems(
                Update,
                (move_view, zoom_view, to_menu_on_return).run_if(in_state(GameState::InGame)),
            );
    }
}

// TODO: not defined yet, using usize as dummy value
struct Cell(usize);

#[derive(Resource, Default)]
struct Game {
    board: Vec<Vec<Cell>>, // board[r][c]
}

impl Game {
    const BOARD_ROW: usize = 10;
    const BOARD_COLUMN: usize = 15;
    const CELL_SIZE: f32 = 30.0;
}

fn setup(
    mut cmds: Commands,
    mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    game.board = (0..Game::BOARD_ROW)
        .map(|r| {
            (0..Game::BOARD_COLUMN)
                .map(|c| {
                    // draw a bigger outer square as boarder
                    let shape_outer =
                        Mesh2dHandle(meshes.add(Rectangle::new(Game::CELL_SIZE, Game::CELL_SIZE)));
                    cmds.spawn(MaterialMesh2dBundle {
                        mesh: shape_outer,
                        material: materials.add(Color::AZURE),
                        transform: Transform::from_xyz(
                            c as f32 * Game::CELL_SIZE,
                            r as f32 * Game::CELL_SIZE,
                            0.0,
                        ),
                        ..default()
                    });

                    // draw a smaller inner square as fill
                    let shape_inner = Mesh2dHandle(
                        meshes.add(Rectangle::new(Game::CELL_SIZE - 1.0, Game::CELL_SIZE - 1.0)),
                    );
                    cmds.spawn(MaterialMesh2dBundle {
                        mesh: shape_inner,
                        material: materials.add(Color::GRAY),
                        transform: Transform::from_xyz(
                            c as f32 * Game::CELL_SIZE,
                            r as f32 * Game::CELL_SIZE,
                            1.0,
                        ),
                        ..default()
                    });

                    Cell(0)
                })
                .collect()
        })
        .collect();
}

fn cleanup() {}

fn move_view(
    mut query_camera: Query<&mut Transform, With<Camera>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let mut camera_tf = query_camera.single_mut();
    let mut d = Vec3::ZERO;
    if input.pressed(KeyCode::KeyW) {
        d.y += 3.0;
    }
    if input.pressed(KeyCode::KeyS) {
        d.y -= 3.0;
    }
    if input.pressed(KeyCode::KeyA) {
        d.x -= 3.0;
    }
    if input.pressed(KeyCode::KeyD) {
        d.x += 3.0;
    }
    camera_tf.translation += d;
}

fn zoom_view(
    mut query_camera: Query<&mut OrthographicProjection, With<Camera>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let mut camera_proj = query_camera.single_mut();
    if input.just_pressed(KeyCode::Equal) {
        // +
        camera_proj.scale /= 1.25;
    }
    if input.just_pressed(KeyCode::Minus) {
        camera_proj.scale *= 1.25;
    }
}

fn to_menu_on_return(
    mut game_state: ResMut<NextState<GameState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Enter) {
        game_state.set(GameState::Menu)
    }
}
