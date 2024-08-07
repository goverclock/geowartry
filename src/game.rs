use bevy::prelude::*;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

use crate::{layer, GlobalState};
mod cell;
mod field_entity;
mod input_event;
mod select_unit;
mod unit;
mod unit_move;
mod view_ctrl;
use layer::Layer;
use unit::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Game>()
            .add_systems(OnEnter(GlobalState::InGame), setup)
            .add_systems(OnExit(GlobalState::InGame), cleanup)
            .add_plugins((
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
                RapierDebugRenderPlugin::default(),
            ))
            .add_plugins((
                unit::unit_plugin,
                unit_move::unit_move_plugin,
                cell::cell_plugin,
                view_ctrl::view_ctrl_plugin,
                select_unit::select_unit_plugin,
                input_event::input_event_plugin,
            ))
            .add_systems(
                Update,
                to_menu_on_return.run_if(in_state(GlobalState::InGame)),
            );
    }
}

#[derive(Resource, Default)]
struct Game;

impl Game {
    const BOARD_ROW: usize = 20;
    const BOARD_COLUMN: usize = 35;
    const CELL_SIZE: f32 = 40.0;
}

fn setup(
    mut rapier_config: ResMut<RapierConfiguration>,
    mut ev_spawn_unit: EventWriter<SpawnUnitEvent>,
) {
    // set gravity to zero
    rapier_config.gravity = Vec2::ZERO;

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
        unit_type: UnitType::Attacker,
        cell_coord: (10, 1),
    });
    // for i in 0..5 {
    //     for j in 0..7 {
    //         ev_spawn_unit.send(SpawnUnitEvent {
    //             unit_type: UnitType::Attacker,
    //             cell_coord: (i, j),
    //         });
    //     }
    // }
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

/// return to menu when press return key
fn to_menu_on_return(
    mut game_state: ResMut<NextState<GlobalState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Enter) {
        game_state.set(GlobalState::Menu)
    }
}

/// convert cell coord(usize, usize) to world transform's xy
fn cell_to_transform(cell_coord: (usize, usize)) -> Vec2 {
    Vec2 {
        x: cell_coord.0 as f32 * Game::CELL_SIZE,
        y: cell_coord.1 as f32 * Game::CELL_SIZE,
    }
}

/// convert world transform's xy to cell coord(col, row)
fn transform_to_cell(tf_xy: Vec2) -> (i64, i64) {
    let x = if tf_xy.x > 0.0 {
        tf_xy.x + Game::CELL_SIZE * 0.5
    } else {
        tf_xy.x - Game::CELL_SIZE * 0.5
    };
    let y = if tf_xy.y > 0.0 {
        tf_xy.y + Game::CELL_SIZE * 0.5
    } else {
        tf_xy.y - Game::CELL_SIZE * 0.5
    };
    ((x / Game::CELL_SIZE) as i64, (y / Game::CELL_SIZE) as i64)
}

/// convert window position to world coords, which then can be used as transform
/// to use this function, add camera_tf: Query<(&Camera, &GlobalTransform)> in
/// the system parameter
fn window_to_world_coords(
    window_pos: Vec2,
    camera_tf: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let (cam, cam_tf) = camera_tf.single();
    cam.viewport_to_world(cam_tf, window_pos)
        .map(|ray| ray.origin.truncate())
}

pub fn despawn_with_component<T: Component>(
    to_despawn: Query<Entity, With<T>>,
    mut cmds: Commands,
) {
    info!("despawn with component: to_despawn={:?}", to_despawn);
    for e in &to_despawn {
        cmds.entity(e).despawn_recursive();
    }
}
