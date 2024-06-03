use bevy::{
    prelude::*,
    render::camera,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};

use crate::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Game>()
            .add_systems(OnEnter(GameState::InGame), setup)
            .add_systems(OnExit(GameState::InGame), cleanup)
            .add_systems(
                Update,
                (
                    mouse_button_input,
                    mouse_move_view,
                    kb_move_view,
                    zoom_view,
                    to_menu_on_return,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

// TODO: not defined yet, using usize as dummy value
struct Cell(usize);

#[derive(Resource, Default)]
struct Game {
    board: Vec<Vec<Cell>>,          // board[r][c]
    right_drag_start: Option<Vec2>, // start of a right mouse drag, None if not dragging
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

fn cleanup(mut query_camera: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>) {
    // reset camera transform and zoom
    let (mut camera_tf, mut camera_proj) = query_camera.single_mut();
    camera_tf.translation = Vec3::ZERO;
    camera_proj.scale = 1.0;
}

fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    mut game: ResMut<Game>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    if buttons.just_pressed(MouseButton::Right) {
        info!("right clicked at {:?}", window.cursor_position());
        game.right_drag_start = window.cursor_position();
    }
    if !buttons.pressed(MouseButton::Right) {
        game.right_drag_start = None;
    }
}

fn mouse_move_view(
    mut query_camera: Query<&mut Transform, With<Camera>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    game: Res<Game>,
) {
    let mut camera_tf = query_camera.single_mut();
    let mut d = Vec3::ZERO;
    let mut move_speed_x = 0.0;
    let mut move_speed_y = 0.0;

    if let Some(start) = game.right_drag_start {
        for e in cursor_moved_events.read() {
            let cur = e.position;
            move_speed_x = (cur.x - start.x) / 10.0;
            move_speed_y = (start.y - cur.y) / 10.0; // note: cursor position has reverse y with bevy coordinates
        }
        move_speed_x = move_speed_x.min(5.0);
        move_speed_x = move_speed_x.max(-5.0);
        move_speed_y = move_speed_y.min(5.0);
        move_speed_y = move_speed_y.max(-5.0);
    }

    d.x += move_speed_x;
    d.y += move_speed_y;
    camera_tf.translation += d;
}

// move view with keyboard(WASD)
fn kb_move_view(
    mut query_camera: Query<&mut Transform, With<Camera>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let mut camera_tf = query_camera.single_mut();
    let mut d = Vec3::ZERO;
    let move_speed = 3.0;
    if input.pressed(KeyCode::KeyW) {
        d.y += move_speed;
    }
    if input.pressed(KeyCode::KeyS) {
        d.y -= move_speed;
    }
    if input.pressed(KeyCode::KeyA) {
        d.x -= move_speed;
    }
    if input.pressed(KeyCode::KeyD) {
        d.x += move_speed;
    }
    camera_tf.translation += d;
}

// note: this function is for debug purpose only
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
