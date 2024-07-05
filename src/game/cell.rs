use super::Game;
use super::Layer;
use crate::GlobalState;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};

/// Cell is **irrelevant** to any entity(except its visual), it just provides
/// information to render the color of the grid, and assist in path finding
/// algorithm.
/// if there is entity(e.g. building, mine) occupying this cell, it just convert
/// itself to matching movable state, it does not manager the entity.
pub struct Cell {
    pub coord: (i64, i64),
    pub state: CellState,
    /// visual entity for inner and outer
    visual: Option<(Entity, Entity)>,
}

/// all the cells has this component, for clean up
#[derive(Component)]
struct CellVisual;

#[derive(Default, Debug, Clone, Copy)]
pub enum CellState {
    #[default]
    Empty, // all else are actually implemented as entities
    Water,
    Iron,
    Rock,
    Building,
    Mine,
}

#[derive(Event, Debug)]
pub struct UpdateCellEvent {
    /// the coord of the cell to update
    pub coord: (i64, i64),
    /// the new state of the cell
    pub new_state: CellState,
}

#[derive(Resource, Default)]
struct CellField(Vec<Vec<Cell>>); // board[c][r]

pub fn cell_plugin(app: &mut App) {
    app.add_event::<UpdateCellEvent>()
        .init_resource::<CellField>()
        .add_systems(OnEnter(GlobalState::InGame), setup)
        .add_systems(
            OnExit(GlobalState::InGame),
            (super::despawn_with_component::<CellVisual>, cleanup),
        )
        .add_systems(Update, cell_state.run_if(in_state(GlobalState::InGame)));

    #[cfg(debug_assertions)]
    app.add_systems(
        Update,
        debug_alter_cell_state.run_if(in_state(GlobalState::InGame)),
    );
}

/// create the cell field when game starts
fn setup(
    mut cmds: Commands,
    mut field: ResMut<CellField>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    field.0 = (0..Game::BOARD_COLUMN as i64)
        .map(|c| {
            (0..Game::BOARD_ROW as i64)
                .map(|r| Cell {
                    coord: (c, r),
                    state: CellState::Empty,
                    visual: None,
                })
                .collect()
        })
        .collect();

    #[cfg(debug_assertions)]
    {
        field.0[2][3].state = CellState::Water;
        field.0[2][4].state = CellState::Water;
        field.0[2][5].state = CellState::Water;
        field.0[3][4].state = CellState::Iron;
        field.0[3][5].state = CellState::Iron;
        field.0[3][6].state = CellState::Iron;
    }

    // and draw the cells
    for r in 0..Game::BOARD_ROW {
        for c in 0..Game::BOARD_COLUMN {
            field.0[c][r].draw(&mut cmds, &mut meshes, &mut materials);
        }
    }
}

fn cleanup(mut field: ResMut<CellField>) {
    // visual entities are despawned using super::despawn_with_component

    field.0 = vec![];
}

impl Cell {
    /// for units to run path finding algorithm
    pub fn is_passable(&self) -> bool {
        use CellState::*;
        match self.state {
            Empty | Mine => true,
            _ => false,
        }
    }

    /// draw/redraw the Cell's visual entity, automatically clears previous
    /// visual entities
    pub fn draw(
        &mut self,
        cmds: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) {
        if let Some((inner, outer)) = self.visual {
            cmds.entity(inner).despawn();
            cmds.entity(outer).despawn();
        }

        let (c, r) = self.coord;
        // draw a bigger outer square as boarder
        let shape_outer = Mesh2dHandle(
            meshes.add(Rectangle::new(Game::CELL_SIZE, Game::CELL_SIZE)),
        );
        let outer = cmds
            .spawn((
                MaterialMesh2dBundle {
                    mesh: shape_outer,
                    material: materials.add(Color::AZURE),
                    transform: Transform::from_xyz(
                        c as f32 * Game::CELL_SIZE,
                        r as f32 * Game::CELL_SIZE,
                        Layer::GameMap.into(),
                    ),
                    ..default()
                },
                CellVisual,
            ))
            .id();

        // draw a smaller inner square as fill
        let shape_inner =
            Mesh2dHandle(meshes.add(Rectangle::new(
                Game::CELL_SIZE - 1.0,
                Game::CELL_SIZE - 1.0,
            )));
        let color = match self.state {
            CellState::Empty => Color::GRAY,
            CellState::Water => Color::BLUE,
            CellState::Iron => Color::BLACK,
            CellState::Rock => Color::TOMATO,
            _ => todo!(),
        };
        let mut inner_z: f32 = Layer::GameMap.into();
        inner_z += 0.1;
        let inner = cmds
            .spawn((
                MaterialMesh2dBundle {
                    mesh: shape_inner,
                    material: materials.add(color),
                    transform: Transform::from_xyz(
                        c as f32 * Game::CELL_SIZE,
                        r as f32 * Game::CELL_SIZE,
                        inner_z,
                    ),
                    ..default()
                },
                CellVisual,
            ))
            .id();

        self.visual = Some((inner, outer));
    }
}

fn cell_state(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut field: ResMut<CellField>,
    mut ev_update_cell: EventReader<UpdateCellEvent>,
) {
    for e in ev_update_cell.read() {
        let (c, r) = e.coord;
        // TODO: to support negative index, implement another way to retrive
        // cell
        let cell = &mut field.0[c as usize][r as usize];
        cell.state = e.new_state;
        cell.draw(&mut cmds, &mut meshes, &mut materials);
    }
}

#[cfg(debug_assertions)]
fn debug_alter_cell_state(
    kb_input: Res<ButtonInput<KeyCode>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_tf: Query<(&Camera, &GlobalTransform)>,
    mut ev_update_cell: EventWriter<UpdateCellEvent>,
) {
    let window = window.single();
    let mut target_state = None;
    if kb_input.pressed(KeyCode::ShiftLeft) {
        target_state = Some(CellState::Iron);
    }
    if kb_input.pressed(KeyCode::ShiftRight) {
        target_state = Some(CellState::Empty);
    }
    if target_state.is_none() {
        return;
    }

    let pos = window.cursor_position();
    if pos.is_none() {
        return;
    }
    let pos = pos.unwrap();
    let world_coord = super::window_to_world_coords(pos, &camera_tf).unwrap();
    let cell_coord = super::transform_to_cell(world_coord);
    if cell_coord.0 < 0 || cell_coord.0 >= super::Game::BOARD_COLUMN as i64 {
        return;
    }
    if cell_coord.1 < 0 || cell_coord.1 >= super::Game::BOARD_ROW as i64 {
        return;
    }
    info!(
        "sending: {:?}",
        UpdateCellEvent {
            coord: cell_coord,
            new_state: target_state.unwrap(),
        }
    );
    ev_update_cell.send(UpdateCellEvent {
        coord: cell_coord,
        new_state: target_state.unwrap(),
    });
}
