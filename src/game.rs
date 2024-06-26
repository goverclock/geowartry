use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};
use bevy_rapier2d::{
    plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

use crate::{layer, GameState};
mod select_area;
mod unit;
mod unit_move;
mod view_ctrl;
use layer::Layer;
use unit::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Game>()
            .add_systems(OnEnter(GameState::InGame), setup)
            .add_systems(OnExit(GameState::InGame), cleanup)
            .add_plugins((
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
                RapierDebugRenderPlugin::default(),
            ))
            .add_plugins((
                unit::unit_plugin,
                unit_move::unit_move_plugin,
                view_ctrl::view_ctrl_plugin,
                select_area::select_area_plugin,
            ))
            .add_systems(
                Update,
                (mouse_button_input, to_menu_on_return)
                    .run_if(in_state(GameState::InGame)),
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
    const CELL_SIZE: f32 = 40.0;
}

fn setup(
    mut cmds: Commands,
    mut game: ResMut<Game>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ev_spawn_unit: EventWriter<SpawnUnitEvent>,
) {
    // set gravity to zero
    rapier_config.gravity = Vec2::ZERO;

    // initialize the board
    game.board = (0..Game::BOARD_ROW)
        .map(|r| {
            (0..Game::BOARD_COLUMN)
                .map(|c| {
                    // draw a bigger outer square as boarder
                    let shape_outer =
                        Mesh2dHandle(meshes.add(Rectangle::new(
                            Game::CELL_SIZE,
                            Game::CELL_SIZE,
                        )));
                    cmds.spawn(MaterialMesh2dBundle {
                        mesh: shape_outer,
                        material: materials.add(Color::AZURE),
                        transform: Transform::from_xyz(
                            c as f32 * Game::CELL_SIZE,
                            r as f32 * Game::CELL_SIZE,
                            Layer::GameMap.into(),
                        ),
                        ..default()
                    });

                    // draw a smaller inner square as fill
                    let shape_inner = Mesh2dHandle(meshes.add(Rectangle::new(
                        Game::CELL_SIZE - 1.0,
                        Game::CELL_SIZE - 1.0,
                    )));
                    let mut inner_z: f32 = Layer::GameMap.into();
                    inner_z += 0.1;
                    cmds.spawn(MaterialMesh2dBundle {
                        mesh: shape_inner,
                        material: materials.add(Color::GRAY),
                        transform: Transform::from_xyz(
                            c as f32 * Game::CELL_SIZE,
                            r as f32 * Game::CELL_SIZE,
                            inner_z,
                        ),
                        ..default()
                    });

                    Cell(())
                })
                .collect()
        })
        .collect();

    // TODO: spawning these random entities for debug purpose
    // generate some units
    // ev_spawn_unit.send(SpawnUnitEvent {
    //     unit_type: UnitType::Attacker,
    //     cell_coord: (0, 0),
    // });
    // ev_spawn_unit.send(SpawnUnitEvent {
    //     unit_type: UnitType::Attacker,
    //     cell_coord: (10, 10),
    // });
    ev_spawn_unit.send(SpawnUnitEvent {
        unit_type: UnitType::Miner,
        cell_coord: (10, 0),
    });
    for i in 0..5 {
        for j in 0..7 {
            ev_spawn_unit.send(SpawnUnitEvent {
                unit_type: UnitType::Attacker,
                cell_coord: (i, j),
            });
        }
    }
}

fn cleanup(
    mut query_camera: Query<
        (&mut Transform, &mut OrthographicProjection),
        With<Camera>,
    >,
) {
    // reset camera transform and zoom
    let (mut camera_tf, mut camera_proj) = query_camera.single_mut();
    camera_tf.translation = Vec3::ZERO;
    camera_proj.scale = 1.0;
}

/// update mouse click status(stored in Game resource) based on mouse input,
/// majorly record the start of a left click or right click
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
    // TODO: Game resource is responsible for recording last inplace left click
    if buttons.just_released(MouseButton::Right) {
        game.right_drag_start = None;
    }
    if buttons.just_released(MouseButton::Left) {
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

/// convert cell coord(usize, usize) to transform's xy
fn cell_to_transform(cell_coord: (usize, usize)) -> Vec2 {
    Vec2 {
        x: cell_coord.0 as f32 * Game::CELL_SIZE,
        y: cell_coord.1 as f32 * Game::CELL_SIZE,
    }
}

/// convert transform's xy to cell coord
#[allow(unused)]
fn transform_to_cell(tf_xy: Vec2) -> (usize, usize) {
    (
        (tf_xy.x / Game::CELL_SIZE) as usize,
        (tf_xy.y / Game::CELL_SIZE) as usize,
    )
}

/// convert window position to world coords, which then can be used as transform
fn window_to_world_coords(
    window_pos: Vec2,
    cam: &Camera,
    cam_tf: &GlobalTransform,
) -> Option<Vec2> {
    cam.viewport_to_world(cam_tf, window_pos)
        .map(|ray| ray.origin.truncate())
}
