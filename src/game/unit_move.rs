use bevy::prelude::*;

use crate::GlobalState;

#[cfg(debug_assertions)]
use super::input_event::DebugSetUnitDestEvent;

#[cfg(debug_assertions)]
#[derive(Resource, Default)]
pub struct DebugTargetCell(pub Option<(i64, i64)>);

pub fn unit_move_plugin(app: &mut App) {
    #[cfg(debug_assertions)]
    app.init_resource::<DebugTargetCell>()
        .add_systems(OnExit(GlobalState::InGame), cleanup)
        .add_systems(
            Update,
            (debug_unit_set_dest, debug_unit_velocity)
                .run_if(in_state(super::GlobalState::InGame)),
        );
}

#[cfg(debug_assertions)]
fn cleanup(mut debug_target_cell: ResMut<DebugTargetCell>) {
    debug_target_cell.0 = None;
}

#[cfg(debug_assertions)]
fn debug_unit_set_dest(
    // mut query_units: Query<(&Selectable, &mut DebugMoveDest), With<Unit>>,
    mut ev_debug_set_unit_dest: EventReader<DebugSetUnitDestEvent>,
    mut target: ResMut<DebugTargetCell>,
) {
    for e in ev_debug_set_unit_dest.read() {
        target.0 = Some(e.0);
        info!("debug target cell={:?}", target.0);
    }
    // if ev_debug_set_unit_dest.is_empty() {
    //     return;
    // }
    // let ev_dest = ev_debug_set_unit_dest.read().next().unwrap().0;
    // for (selected, mut dest) in query_units.iter_mut() {
    //     if !selected.0 {
    //         continue;
    //     }
    //     *dest = DebugMoveDest(Some(ev_dest));
    // }
}

use super::cell::CellField;
use super::Unit;
use bevy_rapier2d::dynamics::ExternalForce;
use bevy_rapier2d::dynamics::Velocity;
#[cfg(debug_assertions)]
fn debug_unit_velocity(
    field: Res<CellField>,
    mut query_unit: Query<
        (&mut Velocity, &mut ExternalForce, &Transform),
        With<Unit>,
    >,
) {
    for (mut v, mut f, tf) in query_unit.iter_mut() {
        // get the cell the unit is in
        let unit_pos = Vec2 {
            x: tf.translation.x,
            y: tf.translation.y,
        };
        let cell_coord = super::transform_to_cell(unit_pos);
        let cell = field.get_cell(cell_coord);
        let cell_center = cell.center();

        // already in target cell, move directly to it
        if cell.debug_distance == 0.0 {
            let center_diff = Vec2 {
                x: cell_center.x - unit_pos.x,
                y: cell_center.y - unit_pos.y,
            };
            v.linvel = center_diff * 5.0;
            f.force = Vec2::ZERO;
            continue;
        }

        // apply cell's direction
        if !cell.debug_direction.is_nan() {
            let dir_v_diff = cell.debug_direction - v.linvel.normalize();
            if !dir_v_diff.is_nan() {
                f.force = (cell.debug_direction + dir_v_diff * 0.0) * 1000.0;
            } else {
                f.force = cell.debug_direction * 1000.0;
            }
        }

        if v.linvel.length() > 200.0 {
            v.linvel = v.linvel.normalize() * 200.0;
        }
    }
}
