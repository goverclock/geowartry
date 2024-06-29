use bevy::{input::mouse::MouseScrollUnit, prelude::*, window::PrimaryWindow};

use super::input_event::MouseStatus;

/// this event is triggered by single left click on a cell, only affects
/// selected units the event is handled by calculating shortes path to the
/// dest cell, stored in its PathDest Component
#[derive(Event)]
struct UnitMoveEvent((usize, usize));

pub fn unit_move_plugin(app: &mut App) {
    // app.add_event::<UnitMoveEvent>().add_systems(
    //     Update,
    //     detect_left_click.run_if(in_state(super::GameState::InGame)),
    // );

    app.add_event::<UnitMoveEvent>();
}

/// detect a in-place left click, then send a UnitMoveEvent
fn detect_left_click(
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_status: Res<MouseStatus>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_tf: Query<(&Camera, &GlobalTransform)>,
    mut ev_unit_move: EventWriter<UnitMoveEvent>,
) {
    let window = window.single();
    if mouse_status.left_drag_start.is_none() {
        return;
    }
    if buttons.just_released(MouseButton::Left)
        && mouse_status.left_drag_start == window.cursor_position()
    {
        info!(
            "in-place left clicked at window: {:?}, sending UnitMoveEvent",
            window.cursor_position().unwrap()
        );
        // TODO: should send cell coord
        let window_pos = window.cursor_position().unwrap();
        let world_coord =
            super::window_to_world_coords(window_pos, &camera_tf).unwrap();
        let cell_coord = super::transform_to_cell(world_coord);
        // TODO: this is inaccurate for now
        info!("{:?}", cell_coord);
        ev_unit_move.send(UnitMoveEvent((cell_coord.0, cell_coord.1)));
    }
}
