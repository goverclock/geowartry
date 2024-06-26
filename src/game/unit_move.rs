use bevy::{prelude::*, window::PrimaryWindow};

/// this event is triggered by single left click on a cell, only affects
/// selected units the event is handled by calculating shortes path to the
/// dest cell, stored in its PathDest Component
#[derive(Event)]
struct UnitMoveEvent((usize, usize));

pub fn unit_move_plugin(app: &mut App) {
    app.add_event::<UnitMoveEvent>().add_systems(
        Update,
        detect_left_click.run_if(in_state(super::GameState::InGame)),
    );
}

/// detect a in-place left click, then send a UnitMoveEvent
fn detect_left_click(
    buttons: Res<ButtonInput<MouseButton>>,
    game: Res<super::Game>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_tf: Query<(&Camera, &GlobalTransform)>,
    mut ev_unit_move: EventWriter<UnitMoveEvent>,
) {
    let window = window.single();
    if game.left_drag_start.is_none() {
        return;
    }
    if buttons.just_released(MouseButton::Left)
        && game.left_drag_start == window.cursor_position()
    {
        info!(
            "in-place left clicked at window: {:?}, sending UnitMoveEvent",
            window.cursor_position().unwrap()
        );
        // TODO: should send cell coord
        let window_pos = window.cursor_position().unwrap();
        let (cam, cam_tf) = camera_tf.single();
        let world_coord =
            super::window_to_world_coords(window_pos, cam, cam_tf).unwrap();
        let cell_coord = super::transform_to_cell(world_coord);
        info!("{:?}", cell_coord);
        ev_unit_move.send(UnitMoveEvent((cell_coord.0, cell_coord.1)));
    }
}