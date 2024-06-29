use bevy::prelude::*;

use super::input_event::MouseStatus;

pub fn view_ctrl_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (mouse_move_view, kb_move_view, zoom_view)
            .run_if(in_state(super::GlobalState::InGame)),
    );
}

/// move view with right drag
pub fn mouse_move_view(
    mut query_camera: Query<&mut Transform, With<Camera>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mouse_status: Res<MouseStatus>,
) {
    let mut camera_tf = query_camera.single_mut();
    let mut d = Vec3::ZERO;
    let mut move_speed_x = 0.0;
    let mut move_speed_y = 0.0;

    if let Some(start) = mouse_status.right_drag_start {
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
pub fn kb_move_view(
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
pub fn zoom_view(
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
