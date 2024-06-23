use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_rapier2d::prelude::*;

use crate::{diep_colors, layer};

/// all units has this component
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
    marker: Unit,
    hp: Health,
    /// ColorMesh2dBundle already contains transform
    color_mesh: ColorMesh2dBundle,
    selectable: Selectable,
    // physics relevant
    collider: Collider,
    rigid_body: RigidBody,
    velocity: Velocity,
    /// must be set to zero
    collider_density: ColliderMassProperties,
    /// actual mass of this unit
    mass: AdditionalMassProperties,
    sleep: Sleeping,
}

#[derive(Debug, Clone, Copy)]
pub enum UnitType {
    Attacker,
    Miner,
}

/// unit module is implemented as plugin, to spawn a unit, just write a SpawnUnitEvent
#[derive(Event, Clone, Copy)]
pub struct SpawnUnitEvent {
    pub unit_type: UnitType,
    pub coord: Vec2,
}
pub fn unit_plugin(app: &mut App) {
    app.add_event::<SpawnUnitEvent>()
        .add_systems(
            Update,
            spawn_unit.run_if(in_state(super::GameState::InGame)),
        )
        .add_systems(OnExit(super::GameState::InGame), cleanup)
        .add_systems(
            Update,
            unit_debug.run_if(in_state(super::GameState::InGame)),
        );
}

fn spawn_unit(
    mut cmds: Commands,
    mut ev_spawn_unit: EventReader<SpawnUnitEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape_circle = Mesh2dHandle(meshes.add(Circle { radius: 30.0 }));
    let color_material_blue = materials.add(diep_colors::DIEP_BLUE);
    let color_material_yellow = materials.add(diep_colors::DIEP_YELLOW);

    for e in ev_spawn_unit.read() {
        match e.unit_type {
            UnitType::Attacker => {
                cmds.spawn((UnitBundle {
                    marker: Unit,
                    hp: Health { max: 10, cur: 10 },
                    selectable: Selectable(false),
                    color_mesh: ColorMesh2dBundle {
                        mesh: shape_circle.clone(),
                        material: color_material_blue.clone(),
                        transform: Transform::from_xyz(
                            e.coord.x,
                            e.coord.y,
                            layer::Layer::Units.into(),
                        ),
                        ..default()
                    },
                    collider: Collider::ball(30.0),
                    rigid_body: RigidBody::Dynamic,
                    velocity: Velocity::linear(Vec2 { x: 10.0, y: 20.0 }),
                    collider_density: ColliderMassProperties::Density(0.0),
                    mass: AdditionalMassProperties::Mass(1.0),
                    sleep: Sleeping::disabled(),
                },));
            }
            UnitType::Miner => {
                cmds.spawn((UnitBundle {
                    marker: Unit,
                    hp: Health { max: 10, cur: 10 },
                    selectable: Selectable(false),
                    color_mesh: ColorMesh2dBundle {
                        mesh: shape_circle.clone(),
                        material: color_material_yellow.clone(),
                        transform: Transform::from_xyz(
                            e.coord.x,
                            e.coord.y,
                            layer::Layer::Units.into(),
                        ),
                        ..default()
                    },
                    collider: Collider::ball(30.0),
                    rigid_body: RigidBody::Fixed,
                    velocity: Velocity::zero(),
                    collider_density: ColliderMassProperties::Density(0.0),
                    mass: AdditionalMassProperties::Mass(1.0),
                    sleep: Sleeping::disabled(),
                },));
            }
        }
    }
}

fn unit_debug(query_units: Query<(&Velocity, Option<&Damping>), With<Unit>>) {
    info!("unit_debug running");
    for u in query_units.iter() {
        info!("{:?}", u);
    }
}

// despawn all units
fn cleanup(mut cmds: Commands, query_units: Query<Entity, With<Unit>>) {
    info!("despawning all units");
    for unit in query_units.iter() {
        cmds.entity(unit).despawn();
    }
}
