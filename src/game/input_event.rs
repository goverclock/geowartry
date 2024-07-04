use bevy::{
    ecs::query::QuerySingleError,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};
use bevy_rapier2d::{pipeline::QueryFilter, plugin::RapierContext};

/// when user in-place left clicked on a unit, select it and unselecte others
#[derive(Event, Debug)]
pub struct SelectSingleUnitEvent(pub Entity);

/// when user in-place left clicked on a cell, order all selected units to move
/// there, contains cell coords(col, row)
#[derive(Event, Debug)]
struct SetUnitDestEvent((i64, i64));

/// TODO: for debug only, works with select_unit::debug_unit_set_dest
/// set a dest for selected unit, which causes the unit move directly to the
/// cell
#[cfg(debug_assertions)]
#[derive(Event, Debug)]
pub struct DebugSetUnitDestEvent(pub (i64, i64));

#[derive(Event, Debug)]
pub struct SelectAreaUnitsEvent(pub SelectArea);

/// the visible rectangle area entity
#[derive(Component, Debug, Clone, Copy)]
pub struct SelectArea {
    /// world coord of select area start
    pub start: Vec2,
    /// world coord of select area end
    pub end: Vec2,
}

#[derive(Resource, Default)]
pub struct MouseStatus {
    /// window position of start of a right mouse drag, None if not dragging
    pub right_drag_start: Option<Vec2>,
    /// window position of start of a left mouse drag, None if not dragging
    pub left_drag_start: Option<Vec2>,
}

impl MouseStatus {
    /// if the drag distance is lower than this, it's treated as a click
    const DRAG_DISTANCE_THRESHOLD: f32 = 10.0;
}

/// this plugin is responsible for generating events that **actually** influence
/// the game(e.g. command to move all selected units, rather than moving the
/// view) based on user inputs
pub fn input_event_plugin(app: &mut App) {
    app.init_resource::<MouseStatus>()
        .add_event::<SelectAreaUnitsEvent>()
        .add_event::<SelectSingleUnitEvent>()
        .add_event::<SetUnitDestEvent>()
        .add_systems(
            Update,
            (mouse_button_input, select_area)
                .run_if(in_state(super::GlobalState::InGame)),
        );
    #[cfg(debug_assertions)]
    app.add_event::<DebugSetUnitDestEvent>();
}

/// update mouse click status(stored in Game resource) based on mouse input,
/// send game commands based on them
fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_status: ResMut<MouseStatus>,
    mut ev_select_single_unit: EventWriter<SelectSingleUnitEvent>,
    mut ev_set_unit_dest: EventWriter<SetUnitDestEvent>,
    #[cfg(debug_assertions)] mut ev_debug_set_unit_dest: EventWriter<
        DebugSetUnitDestEvent,
    >,
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
        && window.cursor_position().is_some()
    {
        let win_pos = window.cursor_position().unwrap();
        let drag_start = mouse_status.left_drag_start.unwrap();
        if win_pos.distance(drag_start) < MouseStatus::DRAG_DISTANCE_THRESHOLD {
            info!("in-place left clicked at window: {:?}", win_pos);

            // check if the click is on a unit
            let point =
                super::window_to_world_coords(win_pos, &camera_tf).unwrap();
            let mut on_unit = false;
            rapier_context.intersections_with_point(
                point,
                QueryFilter::default(),
                |e| {
                    info!(
                        "sending SelectSinglgEvent({:?})",
                        SelectSingleUnitEvent(e)
                    );
                    ev_select_single_unit.send(SelectSingleUnitEvent(e));
                    on_unit = true;
                    false
                },
            );

            // clicked on a cell
            if !on_unit {
                let cell_coord = super::transform_to_cell(point);
                info!("sending {:?}", SetUnitDestEvent(cell_coord));
                ev_set_unit_dest.send(SetUnitDestEvent(cell_coord));
                #[cfg(debug_assertions)]
                {
                    info!("sending {:?}", DebugSetUnitDestEvent(cell_coord));
                    ev_debug_set_unit_dest
                        .send(DebugSetUnitDestEvent(cell_coord));
                }
            }
        }
        mouse_status.left_drag_start = None;
    }
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
    mut ev_select_area: EventWriter<SelectAreaUnitsEvent>,
) {
    // not dragging select, despawn the rectangle area if exist
    if mouse_status.left_drag_start.is_none() {
        match query_select_area.get_single() {
            Ok((e, _, _, area)) => {
                ev_select_area.send(SelectAreaUnitsEvent(*area));
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
    if start_world_coord.distance(cur_world_coord)
        < MouseStatus::DRAG_DISTANCE_THRESHOLD
    {
        return;
    }

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
