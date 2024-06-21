use bevy::{
    ecs::query::QuerySingleError,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};

#[derive(Component, Debug, Clone, Copy)]
pub struct SelectArea {
    /// world coord of select area start
    start: Vec2,
    /// world coord of select area end
    end: Vec2,
}

#[derive(Event)]
pub struct SelectAreaEvent(pub SelectArea);

pub fn select_area_plugin(app: &mut App) {
    app.add_event::<SelectAreaEvent>().add_systems(
        Update,
        (select_area,).run_if(in_state(super::GameState::InGame)),
    );
}

// /// spawn/despawn selected area in the world
// /// any selectable entities colliding with it will be selected
pub fn select_area(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera_tf: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut query_select_area: Query<(Entity, &mut Transform, &mut Mesh2dHandle, &mut SelectArea)>,
    game: ResMut<super::Game>,
    mut ev_select_area: EventWriter<SelectAreaEvent>,
) {
    // not dragging select, despawn the rectangle area if exist
    if game.left_drag_start.is_none() {
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
    let start = game.left_drag_start.unwrap();
    let cur = window.single().cursor_position();
    if cur.is_none() {
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
            *tf = Transform::from_xyz(
                middle_world_coord.x,
                middle_world_coord.y,
                super::Layer::SelectArea.into_z_value(),
            );
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
