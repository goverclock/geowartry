use bevy::prelude::*;

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct Health {
    pub max: usize,
    pub cur: usize,
}

/// a component to mark units that are selectable, the bool value represents if it's selected now
#[derive(Component, Default)]
pub struct Selectable(pub bool);

#[derive(Bundle)]
pub struct UnitBundle {
    pub marker: Unit,
    pub hp: Health,
    /// ColorMesh2dBundle already contains transform
    pub color_mesh: ColorMesh2dBundle,
    pub selectable: Selectable,
}
