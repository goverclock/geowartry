use bevy::prelude::*;

#[cfg(debug_assertions)]
use super::input_event::DebugSetUnitDestEvent;

#[cfg(debug_assertions)]
#[derive(Resource, Default)]
pub struct DebugTargetCell(Option<(i64, i64)>);

pub fn unit_move_plugin(app: &mut App) {
    #[cfg(debug_assertions)]
    app.init_resource::<DebugTargetCell>().add_systems(
        Update,
        debug_unit_set_dest.run_if(in_state(super::GlobalState::InGame)),
    );
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
