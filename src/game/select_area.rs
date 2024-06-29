use bevy::{
    ecs::query::QuerySingleError,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};

use super::{input_event::MouseStatus, Selectable, Unit};

/// the visible rectangle area entity
#[derive(Component, Debug, Clone, Copy)]
struct SelectArea {
    /// world coord of select area start
    start: Vec2,
    /// world coord of select area end
    end: Vec2,
}

#[derive(Event)]
struct SelectAreaEvent(SelectArea);

pub fn select_area_plugin(app: &mut App) {
    app.add_event::<SelectAreaEvent>().add_systems(
        Update,
        (select_area, select_units)
            .run_if(in_state(super::GlobalState::InGame)),
    );
}

/// spawn/despawn selected area in the world
/// any selectable entities colliding with it will be selected
fn select_area(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera_tf: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut query_select_area: Query<(
        Entity,
        &mut Transform,
        &mut Mesh2dHandle,
        &mut SelectArea,
    )>,
    mouse_status: ResMut<MouseStatus>,
    mut ev_select_area: EventWriter<SelectAreaEvent>,
) {
    // not dragging select, despawn the rectangle area if exist
    if mouse_status.left_drag_start.is_none() {
        match query_select_area.get_single() {
            Ok((e, _, _, area)) => {
                ev_select_area.send(SelectAreaEvent(*area));
                cmds.entity(e).despawn();
                info!("select area({:?}) despawned", area);
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
    let start = mouse_status.left_drag_start.unwrap();
    let cur = window.single().cursor_position();
    if cur.is_none() || start == cur.unwrap() {
        return;
    }
    let cur = cur.unwrap();

    // turn drag start/end window position into world coords
    let start = super::window_to_world_coords(start, &camera_tf);
    let cur = super::window_to_world_coords(cur, &camera_tf);
    if start.is_none() || cur.is_none() {
        info!(
            "cursor position to world is None, not handled for selecting area"
        );
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
    let shape =
        Mesh2dHandle(meshes.add(Rectangle::new(area_length, area_width)));

    // update transform of the rect if it exists,
    // else spawn it
    match query_select_area.get_single_mut() {
        Ok((_, mut tf, mut mesh, mut area)) => {
            *tf = Transform::from_xyz(
                middle_world_coord.x,
                middle_world_coord.y,
                super::Layer::SelectArea.into(),
            );
            *mesh = shape;
            area.start = start_world_coord;
            area.end = cur_world_coord;
        }
        Err(QuerySingleError::NoEntities(_)) => {
            static SELECT_AREA_COLOR: Color = Color::rgba(0.0, 0.3, 0.0, 0.5);
            cmds.spawn((
                MaterialMesh2dBundle {
                    mesh: shape,
                    material: materials.add(SELECT_AREA_COLOR), // color of select area, semi transparent
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

/// mark selectable units as selected if they are in select area
fn select_units(
    mut query_units: Query<(&mut Selectable, &Transform), With<Unit>>,
    mut ev_select_area: EventReader<SelectAreaEvent>,
) {
    if ev_select_area.is_empty() {
        return;
    }

    let area = ev_select_area.read().next().unwrap().0;
    let select_up = area.start.y.max(area.end.y);
    let select_down = area.start.y.min(area.end.y);
    let select_left = area.start.x.min(area.end.x);
    let select_right = area.start.x.max(area.end.x);
    for (mut select, tf) in query_units.iter_mut() {
        // info!("x={} y={}", tf.translation.x, tf.translation.y);
        let unit_x = tf.translation.x;
        let unit_y = tf.translation.y;
        if unit_x <= select_right
            && unit_x >= select_left
            && unit_y <= select_up
            && unit_y >= select_down
        {
            select.0 = true;
            info!("selected unit at {:?}", tf.translation);
        } else if select.0 {
            select.0 = false;
            info!("unselected unit at {:?}", tf.translation);
        }
    }
}
