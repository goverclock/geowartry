use bevy::prelude::*;

use super::input_event::{SelectAreaUnitsEvent, SelectSingleUnitEvent};
use super::{Selectable, Unit};

pub fn select_unit_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (select_area_units, select_single_unit)
            .run_if(in_state(super::GlobalState::InGame)),
    );
}

/// mark selectable units as selected if they are in select area
fn select_area_units(
    mut query_units: Query<(&mut Selectable, &Transform), With<Unit>>,
    mut ev_select_area: EventReader<SelectAreaUnitsEvent>,
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

/// select the single unit, and unselect all other units
fn select_single_unit(
    mut query_units: Query<(Entity, &mut Selectable), With<Unit>>,
    mut ev_select_single_unit: EventReader<SelectSingleUnitEvent>,
) {
    if ev_select_single_unit.is_empty() {
        return;
    }
    let selected_unit_id = ev_select_single_unit.read().next().unwrap().0;
    for (id, mut selected) in query_units.iter_mut() {
        if id == selected_unit_id {
            selected.0 = true;
            info!("selected unit {:?}", id);
        } else if selected.0 {
            selected.0 = false;
            info!("unselected unit {:?}", id);
        }
    }
}
