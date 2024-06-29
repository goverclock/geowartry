use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::{pipeline::QueryFilter, plugin::RapierContext};

use crate::game::window_to_world_coords;

#[derive(Resource, Default)]
pub struct MouseStatus {
    /// window position of start of a right mouse drag, None if not dragging
    pub right_drag_start: Option<Vec2>,
    /// window position of start of a left mouse drag, None if not dragging
    pub left_drag_start: Option<Vec2>,
}

/// this plugin is responsible for generating events that **actually** influence
/// the game(e.g. command to move all selected units, rather than moving the
/// view) based on user inputs
/// TODO: the only exception is select_area, which can also be integrated here
pub fn input_event_plugin(app: &mut App) {
    app.init_resource::<MouseStatus>()
        .add_event::<SelectSingleEvent>()
        .add_systems(
            Update,
            mouse_button_input.run_if(in_state(super::GlobalState::InGame)),
        );
}

/// when user in-place left clicked on a unit, select it and unselecte others
#[derive(Event)]
struct SelectSingleEvent(Entity);

#[derive(Event)]
struct SetDestEvent((i64, i64));

/// update mouse click status(stored in Game resource) based on mouse input,
/// send game commands based on them
fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_status: ResMut<MouseStatus>,
    mut ev_select_single: EventWriter<SelectSingleEvent>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_tf: Query<(&Camera, &GlobalTransform)>,
    rapier_context: Res<RapierContext>,
) {
    let window = window.single();
    if buttons.just_pressed(MouseButton::Right) {
        info!("right pressed at window: {:?}", window.cursor_position());
        mouse_status.right_drag_start = window.cursor_position();
    }
    if buttons.just_pressed(MouseButton::Left) {
        info!("left pressed at window: {:?}", window.cursor_position());
        mouse_status.left_drag_start = window.cursor_position();
    }
    if buttons.just_released(MouseButton::Right)
        && mouse_status.right_drag_start.is_some()
    {
        mouse_status.right_drag_start = None;
    }
    if buttons.just_released(MouseButton::Left)
        && mouse_status.left_drag_start.is_some()
    {
        if window.cursor_position() == mouse_status.left_drag_start {
            info!(
                "in-place left clicked at window: {:?}",
                window.cursor_position()
            );
            let point = window_to_world_coords(
                window.cursor_position().unwrap(),
                &camera_tf,
            );
            rapier_context.intersections_with_point(
                point.unwrap(),
                QueryFilter::default(),
                |x| {
                    info!("sending SelectSinglgEvent({:?})", x);
                    ev_select_single.send(SelectSingleEvent(x));
                    false
                },
            );
        }
        mouse_status.left_drag_start = None;
    }
}
