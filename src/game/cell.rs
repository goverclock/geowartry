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
    /// distance of this cell to debug target cell
    #[cfg(debug_assertions)]
    debug_distance: f32,
    #[cfg(debug_assertions)]
    debug_visual: Option<Entity>,
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

#[cfg(debug_assertions)]
#[derive(Event)]
struct DebugRedrawEvent;

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
    app.add_event::<DebugRedrawEvent>().add_systems(
        Update,
        (
            debug_alter_cell_state,
            debug_calculate_distance,
            debug_visual_redraw,
        )
            .run_if(in_state(GlobalState::InGame)),
    );
}

/// create the cell field when game starts
fn setup(
    cmds: Commands,
    mut field: ResMut<CellField>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    field.0 = (0..Game::BOARD_COLUMN as i64)
        .map(|c| {
            (0..Game::BOARD_ROW as i64)
                .map(|r| Cell {
                    coord: (c, r),
                    state: CellState::Empty,
                    visual: None,
                    #[cfg(debug_assertions)]
                    debug_distance: 0.0,
                    #[cfg(debug_assertions)]
                    debug_visual: None,
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
    field.draw_all(cmds, meshes, materials);
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

        #[cfg(debug_assertions)]
        self.debug_draw(cmds, meshes, materials);
    }

    /// will only be called at the end of draw()
    #[cfg(debug_assertions)]
    pub fn debug_draw(
        &mut self,
        cmds: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) {
        if let Some(dv) = self.debug_visual {
            cmds.entity(dv).despawn();
        }
        let (c, r) = self.coord;
        let distance_visual = cmds
            .spawn((
                Text2dBundle {
                    text: Text {
                        sections: vec![TextSection::new(
                            format!("{:.1}", self.debug_distance),
                            TextStyle {
                                font_size: 15.0,
                                color: if self.debug_distance == 0.0 {
                                    Color::RED
                                } else {
                                    Color::BLACK
                                },
                                ..default()
                            },
                        )],
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        c as f32 * Game::CELL_SIZE,
                        r as f32 * Game::CELL_SIZE,
                        Layer::Debug.into(),
                    ),
                    text_anchor: bevy::sprite::Anchor::TopCenter,
                    ..default()
                },
                CellVisual,
            ))
            .id();
        self.debug_visual = Some(distance_visual);
    }
}

impl CellField {
    /// draw/redraw all the Cell's visual entity
    fn draw_all(
        &mut self,
        mut cmds: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        info!("draw_all");
        for r in 0..Game::BOARD_ROW {
            for c in 0..Game::BOARD_COLUMN {
                self.0[c][r].draw(&mut cmds, &mut meshes, &mut materials);
            }
        }
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

/// use left&right shift to change a cell to empty or iron
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

/// calculate all cell's distance to the debug target cell using BFS
use super::input_event::DebugSetUnitDestEvent;
#[cfg(debug_assertions)]
fn debug_calculate_distance(
    mut field: ResMut<CellField>,
    mut ev_debug_set_unit_dest: EventReader<DebugSetUnitDestEvent>,
    mut ev_debug_redraw: EventWriter<DebugRedrawEvent>,
) {
    if ev_debug_set_unit_dest.is_empty() {
        return;
    }
    let dest = ev_debug_set_unit_dest.read().next().unwrap().0;

    // reset previous results
    info!("reset distances");
    for c in 0..field.0.len() {
        for r in 0..field.0[0].len() {
            field.0[c][r].debug_distance =
                (Game::BOARD_COLUMN + Game::BOARD_ROW) as f32;
        }
    }
    field.0[dest.0 as usize][dest.1 as usize].debug_distance = 0.0;

    use std::collections::VecDeque;
    let dx = [0, 1, 0, -1, -1, -1, 1, 1];
    let dy = [1, 0, -1, 0, 1, -1, 1, -1];
    let mut q: VecDeque<(i64, i64, f32)> = VecDeque::new();
    q.push_back((dest.0, dest.1, 0.0));
    while !q.is_empty() {
        let (x, y, distance) = q.pop_front().unwrap();
        for i in 0..8 {
            let nx = x + dx[i];
            let ny = y + dy[i];
            if nx < 0
                || nx >= Game::BOARD_COLUMN as i64
                || ny < 0
                || ny >= Game::BOARD_ROW as i64
            {
                continue;
            }
            let new_distance =
                distance + ((dx[i] * dx[i] + dy[i] * dy[i]) as f32).sqrt();
            if !field.0[nx as usize][ny as usize].is_passable() {
                continue;
            }
            // avoid moving diagonally through two obstacles
            if dx[i] * dy[i] != 0 {
                if !field.0[x as usize][ny as usize].is_passable()
                    && !field.0[nx as usize][y as usize].is_passable()
                {
                    continue;
                }
            }

            let ncell = &mut field.0[nx as usize][ny as usize];
            if new_distance < ncell.debug_distance {
                ncell.debug_distance = new_distance;
                q.push_back((nx, ny, new_distance));
            }
        }
    }
    ev_debug_redraw.send(DebugRedrawEvent);
}

#[cfg(debug_assertions)]
fn debug_visual_redraw(
    mut ev_debug_redraw: EventReader<DebugRedrawEvent>,
    mut field: ResMut<CellField>,
    cmds: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    for _ in ev_debug_redraw.read() {
        field.draw_all(cmds, meshes, materials);
        return;
    }
}
