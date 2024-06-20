use bevy::{
    ecs::query::QuerySingleError,
    prelude::*,
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
                    select_area,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

/// TODO: not defined yet
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

                    Cell(())
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

/// move view with right drag
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

/// move view with keyboard(WASD)
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

/// use = and - key to zoom in and out camera view
/// ### Note: this function is for debug purpose only
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

/// return to menu when press return key
fn to_menu_on_return(
    mut game_state: ResMut<NextState<GameState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Enter) {
        game_state.set(GameState::Menu)
    }
}

#[derive(Component, Debug)]
struct SelectArea {
    /// world coord of select area start
    start: Vec2,
    /// world coord of select area end
    end: Vec2,
}

// /// spawn/despawn selected area in the world
// /// any selectable entities colliding with it will be selected
fn select_area(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera_tf: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut query_select_area: Query<(Entity, &mut Transform, &mut Mesh2dHandle, &mut SelectArea)>,
    game: ResMut<Game>,
) {
    // not dragging select, despawn the rectangle area if exist
    if let None = game.left_drag_start {
        match query_select_area.get_single() {
            Ok((e, _, _, area)) => {
                info!("selected area {:?} ", area);
                cmds.entity(e).despawn();
            }
            Err(QuerySingleError::NoEntities(_)) => {}
            Err(QuerySingleError::MultipleEntities(_)) => {
                unreachable!("should only have at most one select area")
            }
        }
        return;
    }

    // dragging
    // get drag start and current window position
    let start = game.left_drag_start.unwrap();
    let cur = window.single().cursor_position();
    if let None = cur {
        return;
    }
    let cur = cur.unwrap();

    // turn drag start/end window position into world coords
    let (cam, cam_tf) = camera_tf.single();
    let start = window_to_world_coords(start, cam, cam_tf);
    let cur = window_to_world_coords(cur, cam, cam_tf);
    if start.is_none() || cur.is_none() {
        info!("cursor position to world is None, not handled for selecting area");
    }
    let start_world_coord = start.unwrap();
    let cur_world_coord = cur.unwrap();

    // the transform of a rect is the center of it
    let middle_world_coord = Vec2::new(
        (cur_world_coord.x + start_world_coord.x) / 2.0,
        (cur_world_coord.y + start_world_coord.y) / 2.0,
    );
    let area_length = (start_world_coord.x - cur_world_coord.x).abs();
    let area_width = (start_world_coord.y - cur_world_coord.y).abs();
    let shape = Mesh2dHandle(meshes.add(Rectangle::new(area_length, area_width)));

    // update transform of the rect if it exists,
    // else spawn it
    match query_select_area.get_single_mut() {
        Ok((_, mut tf, mut mesh, mut area)) => {
            *tf = Transform::from_xyz(middle_world_coord.x, middle_world_coord.y, 2.0);
            *mesh = shape;
            area.start = start_world_coord;
            area.end = cur_world_coord;
        }
        Err(QuerySingleError::NoEntities(_)) => {
            cmds.spawn((
                MaterialMesh2dBundle {
                    mesh: shape,
                    material: materials.add(Color::rgba(0.0, 0.3, 0.0, 0.5)), // color of select area, semi transparent
                    ..default()
                },
                SelectArea {
                    start: start_world_coord,
                    end: cur_world_coord,
                },
            ));
            info!("select area spawned");
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            unreachable!("should only have at most one select area")
        }
    }
}

fn window_to_world_coords(
    window_pos: Vec2,
    cam: &Camera,
    cam_tf: &GlobalTransform,
) -> Option<Vec2> {
    cam.viewport_to_world(cam_tf, window_pos)
        .map(|ray| ray.origin.truncate())
}
