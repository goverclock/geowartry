use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_rapier2d::prelude::*;

use crate::{diep_colors, layer};

const CIRCLE_RADIUS: f32 = super::Game::CELL_SIZE * 0.45;
const SQUARE_HALF: f32 = super::Game::CELL_SIZE * 0.4;

/// all units has this component
#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct Health {
    pub max: usize,
    pub cur: usize,
}

/// a component to mark units that are selectable, the bool value represents if
/// it's selected now
#[derive(Component)]
pub struct Selectable(pub bool);

/// TODO: for debug only, works with input_event::DebugUnitMoveEvent
#[derive(Component)]
pub struct DebugMoveDest(pub Option<(i64, i64)>);

#[derive(Bundle)]
struct Physics2dBundle {
    collider: Collider,
    rigid_body: RigidBody,
    velocity: Velocity,
    /// must be set to zero
    collider_density: ColliderMassProperties,
    /// actual mass of this unit
    mass: AdditionalMassProperties,
    force: ExternalForce,
    sleep: Sleeping,
}

/// this doesn't include physics, add Physics2dbundle for that
#[derive(Bundle)]
pub struct UnitBundle {
    marker: Unit,
    hp: Health,
    /// ColorMesh2dBundle already contains transform
    color_mesh: ColorMesh2dBundle,
    selectable: Selectable,
}

#[derive(Debug, Clone, Copy)]
pub enum UnitType {
    Attacker,
    Miner,
}

/// unit module is implemented as plugin, to spawn a unit, just write a
/// SpawnUnitEvent
#[derive(Event, Clone, Copy)]
pub struct SpawnUnitEvent {
    pub unit_type: UnitType,
    /// the coord in the Cell, not the transform
    pub cell_coord: (usize, usize),
}

/// this plugin provides interface to spawn new units(the [`SpawnUnitEvent`]),
/// it also do some clear up work on exit [`super::GlobalState::InGame`] by
/// despawning all units
pub fn unit_plugin(app: &mut App) {
    app.add_event::<SpawnUnitEvent>()
        .add_systems(
            Update,
            spawn_unit.run_if(in_state(super::GlobalState::InGame)),
        )
        .add_systems(OnExit(super::GlobalState::InGame), cleanup);
}

fn spawn_unit(
    mut cmds: Commands,
    mut ev_spawn_unit: EventReader<SpawnUnitEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape_circle = Mesh2dHandle(meshes.add(Circle {
        radius: CIRCLE_RADIUS,
    }));
    let shape_square = Mesh2dHandle(meshes.add(Rectangle {
        half_size: Vec2 {
            x: SQUARE_HALF,
            y: SQUARE_HALF,
        },
    }));
    let color_material_blue = materials.add(diep_colors::DIEP_BLUE);
    let color_material_yellow = materials.add(diep_colors::DIEP_YELLOW);

    for e in ev_spawn_unit.read() {
        let tf_coord = super::cell_to_transform(e.cell_coord);
        match e.unit_type {
            UnitType::Attacker => {
                cmds.spawn((
                    UnitBundle {
                        marker: Unit,
                        hp: Health { max: 10, cur: 10 },
                        selectable: Selectable(false),
                        color_mesh: ColorMesh2dBundle {
                            mesh: shape_circle.clone(),
                            material: color_material_blue.clone(),
                            transform: Transform::from_xyz(
                                tf_coord.x,
                                tf_coord.y,
                                layer::Layer::Units.into(),
                            ),
                            ..default()
                        },
                    },
                    Physics2dBundle {
                        collider: Collider::ball(CIRCLE_RADIUS),
                        rigid_body: RigidBody::Dynamic,
                        // velocity: Velocity::linear(Vec2 { x: 10.0, y: 20.0 }),
                        velocity: Velocity::zero(),
                        collider_density: ColliderMassProperties::Density(0.0),
                        mass: AdditionalMassProperties::Mass(1.0),
                        force: ExternalForce {
                            force: Vec2::ZERO,
                            torque: 0.0,
                        },
                        sleep: Sleeping::disabled(),
                    },
                    DebugMoveDest(None),
                ));
            }
            UnitType::Miner => {
                cmds.spawn((
                    UnitBundle {
                        marker: Unit,
                        hp: Health { max: 10, cur: 10 },
                        selectable: Selectable(false),
                        color_mesh: ColorMesh2dBundle {
                            mesh: shape_square.clone(),
                            material: color_material_yellow.clone(),
                            transform: Transform::from_xyz(
                                tf_coord.x,
                                tf_coord.y,
                                layer::Layer::Units.into(),
                            ),
                            ..default()
                        },
                    },
                    Physics2dBundle {
                        collider: Collider::cuboid(SQUARE_HALF, SQUARE_HALF),
                        rigid_body: RigidBody::Dynamic,
                        velocity: Velocity::zero(),
                        collider_density: ColliderMassProperties::Density(0.0),
                        mass: AdditionalMassProperties::Mass(1.0),
                        force: ExternalForce {
                            force: Vec2::ZERO,
                            torque: 0.0,
                        },
                        sleep: Sleeping::disabled(),
                    },
                ));
            }
        }
    }
}

#[allow(unused)]
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
