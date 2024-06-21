use crate::{layer, GameState};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};

mod select_area;
mod unit;
mod view_ctrl;
use layer::Layer;
use unit::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Game>()
            .add_systems(OnEnter(GameState::InGame), setup)
            .add_systems(OnExit(GameState::InGame), cleanup)
            .add_plugins(view_ctrl::view_ctrl_plugin)
            .add_plugins(select_area::select_area_plugin)
            .add_systems(
                Update,
                (mouse_button_input, to_menu_on_return).run_if(in_state(GameState::InGame)),
            );
    }
}

struct Cell(());

#[derive(Resource, Default)]
struct Game {
    board: Vec<Vec<Cell>>, // board[r][c]
    /// window position of start of a right mouse drag, None if not dragging
    right_drag_start: Option<Vec2>,
    /// window position of start of a left mouse drag, None if not dragging
    left_drag_start: Option<Vec2>,
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
    // initialize the board
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
                            Layer::GameMap.into_z_value(),
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
                            Layer::GameMap.into_z_value() + 0.1,
                        ),
                        ..default()
                    });

                    Cell(())
                })
                .collect()
        })
        .collect();

    // generate some units
    let shape = Mesh2dHandle(meshes.add(Circle { radius: 50.0 }));
    cmds.spawn(unit::UnitBundle {
        hp: Health { max: 10, cur: 10 },
        marker: Selectable(false),
        color_mesh: ColorMesh2dBundle {
            mesh: shape,
            material: materials.add(Color::BLUE),
            transform: Transform::from_xyz(0.0, 0.0, Layer::Units.into_z_value()),
            ..default()
        },
    });
}

fn cleanup(mut query_camera: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>) {
    // reset camera transform and zoom
    let (mut camera_tf, mut camera_proj) = query_camera.single_mut();
    camera_tf.translation = Vec3::ZERO;
    camera_proj.scale = 1.0;
}

/// update mouse click status(stored in Game resource) based on mouse input
fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    mut game: ResMut<Game>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    if buttons.just_pressed(MouseButton::Right) {
        info!("right clicked at window: {:?}", window.cursor_position());
        game.right_drag_start = window.cursor_position();
    }
    if buttons.just_pressed(MouseButton::Left) {
        info!("left clicked at window: {:?}", window.cursor_position());
        game.left_drag_start = window.cursor_position();
    }
    if !buttons.pressed(MouseButton::Right) {
        game.right_drag_start = None;
    }
    if !buttons.pressed(MouseButton::Left) {
        game.left_drag_start = None;
    }
}

/// return to menu when press return key
fn to_menu_on_return(
    mut game_state: ResMut<NextState<GameState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Enter) {
        game_state.set(GameState::Menu)
    }
}
