use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    max: usize,
    cur: usize,
}

#[derive(Component)]
pub struct Selectable;

#[derive(Bundle)]
struct UnitBundle {
    hp: Health,
    /// ColorMesh2dBundle already contains transform
    color_mesh: ColorMesh2dBundle,
    marker: Selectable,
}
