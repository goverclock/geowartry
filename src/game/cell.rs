use core::panic;

use super::Game;
use super::Layer;
use crate::GlobalState;
use bevy::ecs::reflect::ReflectCommandExt;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};
use bevy_rapier2d::parry::query::details::closest_points_ball_convex_polyhedron;

/// Cell is **irrelevant** to any entity(except its visual), it just provides
/// information to render the color of the grid, and assist in path finding
/// algorithm.
/// if there is entity(e.g. building, mine) occupying this cell, it just convert
/// itself to matching movable state, it does not manager the entity.
#[derive(Debug)]
pub struct Cell {
    pub coord: (i64, i64),
    pub state: CellState,
    /// visual entity for inner and outer
    visual: Option<(Entity, Entity)>,
    /// distance of this cell to debug target cell
    #[cfg(debug_assertions)]
    pub debug_distance: f32,
    #[cfg(debug_assertions)]
    debug_distance_visual: Option<Entity>,
    #[cfg(debug_assertions)]
    pub debug_direction: Vec2,
    #[cfg(debug_assertions)]
    debug_direction_visual: Option<Entity>,
}

/// all the cells has this component, for clean up
#[derive(Component)]
struct CellVisual;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
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
pub struct CellField(Vec<Vec<Cell>>); // board[c][r]

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
        );
    // .add_systems(
    //     Update,
    //     update_cell_state.run_if(in_state(GlobalState::InGame)),
    // );

    #[cfg(debug_assertions)]
    app.add_event::<DebugRedrawEvent>().add_systems(
        Update,
        (
            debug_alter_cell_state,
            update_cell_state, // TODO: not debug, add to above
            debug_calculate_distance,
            debug_calculate_direction,
            debug_visual_redraw,
        )
            .chain()
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
    field.init();

    #[cfg(debug_assertions)]
    {
        // field.get_cell_mut((2, 3)).state = CellState::Water;
        // field.get_cell_mut((2, 4)).state = CellState::Water;
        // field.get_cell_mut((2, 5)).state = CellState::Water;
        // field.get_cell_mut((3, 4)).state = CellState::Iron;
        // field.get_cell_mut((3, 5)).state = CellState::Iron;
        // field.get_cell_mut((3, 6)).state = CellState::Iron;
    }

    // and draw the cells
    field.draw_all(cmds, meshes, materials);
}

fn cleanup(mut field: ResMut<CellField>) {
    // visual entities are despawned using super::despawn_with_component

    field.clear();
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

    /// the transform of the center of this cell
    pub fn center(&self) -> Vec2 {
        Vec2 {
            x: self.coord.0 as f32 * Game::CELL_SIZE,
            y: self.coord.1 as f32 * Game::CELL_SIZE,
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
        self.debug_draw(cmds);
    }

    /// will only be called at the end of draw()
    #[cfg(debug_assertions)]
    pub fn debug_draw(&mut self, cmds: &mut Commands) {
        let (c, r) = self.coord;

        if let Some(dist_visual) = self.debug_distance_visual {
            cmds.entity(dist_visual).despawn();
            self.debug_distance_visual = None;
        }
        if let Some(dire_visual) = self.debug_direction_visual {
            cmds.entity(dire_visual).despawn();
            self.debug_direction_visual = None;
        }
        if !self.is_passable() {
            return;
        }

        // distance visual
        let mut s = format!("{:.1}", self.debug_distance);
        if self.debug_distance == f32::MAX {
            s = String::from("inf");
        }
        let distance_visual = cmds
            .spawn((
                Text2dBundle {
                    text: Text {
                        sections: vec![TextSection::new(
                            s,
                            TextStyle {
                                font_size: 15.0,
                                color: if self.debug_distance == 0.0 {
                                    Color::GREEN
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
        self.debug_distance_visual = Some(distance_visual);

        // direction visual
        let arrow = "-->".to_string();
        let mut tf = Transform::from_xyz(
            c as f32 * Game::CELL_SIZE,
            r as f32 * Game::CELL_SIZE,
            Layer::Debug.into(),
        );
        let angle = self.debug_direction.y.atan2(self.debug_direction.x);
        tf.rotate(Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), angle));
        let direction_visual = cmds
            .spawn((
                Text2dBundle {
                    text: Text {
                        sections: vec![TextSection::new(
                            arrow,
                            TextStyle {
                                font_size: 20.0,
                                color: Color::ALICE_BLUE,
                                ..default()
                            },
                        )],
                        ..default()
                    },
                    transform: tf,
                    text_anchor: bevy::sprite::Anchor::Center,
                    ..default()
                },
                CellVisual,
            ))
            .id();
        self.debug_direction_visual = Some(direction_visual);
    }
}

impl CellField {
    /// create a cell field of [`Game::BOARD_ROW`] rows and
    /// [`Game::BOARD_COLUMN`] columns
    fn init(&mut self) {
        self.0 = (0..Game::BOARD_COLUMN as i64)
            .map(|c| {
                (0..Game::BOARD_ROW as i64)
                    .map(|r| Cell {
                        coord: (c, r),
                        state: CellState::Empty,
                        visual: None,
                        #[cfg(debug_assertions)]
                        debug_distance: f32::NAN,
                        #[cfg(debug_assertions)]
                        debug_distance_visual: None,
                        #[cfg(debug_assertions)]
                        debug_direction: Vec2::NAN,
                        #[cfg(debug_assertions)]
                        debug_direction_visual: None,
                    })
                    .collect()
            })
            .collect();
    }

    /// TODO: should support negative coords
    pub fn get_cell(&self, cell_coord: (i64, i64)) -> &Cell {
        let (c, r) = cell_coord;
        if c < Self::min_col() || c > Self::max_col() {
            panic!("invalid column index: {c}");
        }
        if r < Self::min_row() || r > Self::max_row() {
            panic!("invalid row index: {r}");
        }
        &self.0[c as usize][r as usize]
    }

    pub fn get_cell_mut(&mut self, cell_coord: (i64, i64)) -> &mut Cell {
        let (c, r) = cell_coord;
        if c < Self::min_col() || c > Self::max_col() {
            panic!("invalid column index: {c}");
        }
        if r < Self::min_row() || r > Self::max_row() {
            panic!("invalid row index: {r}");
        }
        &mut self.0[c as usize][r as usize]
    }

    fn get_reachable_adjacent_cells(&self, cell: &Cell) -> Vec<&Cell> {
        let (c, r) = cell.coord;
        let mut ret = vec![];
        let dc = [0, 1, 0, -1, -1, -1, 1, 1];
        let dr = [1, 0, -1, 0, 1, -1, 1, -1];
        for i in 0..8 {
            let nc = c + dc[i];
            let nr = r + dr[i];

            // cell out of field is not reachable
            if nc < Self::min_col()
                || nc > Self::max_col()
                || nr < Self::min_row()
                || nr > Self::max_row()
            {
                continue;
            }

            let ncell = self.get_cell((nc, nr));
            // obstacle is not reachable
            if !ncell.is_passable() {
                continue;
            }

            // cell blocked by diagonal obstacles is not reachable
            if (nc - c) * (nr - r) != 0
                && !self.get_cell((nc, r)).is_passable()
                && !self.get_cell((c, nr)).is_passable()
            {
                continue;
            }
            ret.push(self.get_cell((nc, nr)));
        }
        ret
    }

    pub fn get_adjacent_obstacles(&self, cell: &Cell) -> Vec<&Cell> {
        let (c, r) = cell.coord;
        let mut ret = vec![];
        let dc = [0, 1, 0, -1, -1, -1, 1, 1];
        let dr = [1, 0, -1, 0, 1, -1, 1, -1];
        for i in 0..8 {
            let nc = c + dc[i];
            let nr = r + dr[i];

            if nc < Self::min_col()
                || nc > Self::max_col()
                || nr < Self::min_row()
                || nr > Self::max_row()
            {
                continue;
            }

            let ncell = self.get_cell((nc, nr));
            if !ncell.is_passable() {
                ret.push(ncell);
            }
        }
        ret
    }

    /// draw/redraw all the Cell's visual entity
    fn draw_all(
        &mut self,
        mut cmds: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        info!("draw_all");
        for r in 0..Game::BOARD_ROW as i64 {
            for c in 0..Game::BOARD_COLUMN as i64 {
                self.get_cell_mut((c, r)).draw(
                    &mut cmds,
                    &mut meshes,
                    &mut materials,
                );
            }
        }
    }

    fn min_col() -> i64 {
        0
    }

    fn max_col() -> i64 {
        Game::BOARD_COLUMN as i64 - 1
    }

    fn min_row() -> i64 {
        0
    }

    fn max_row() -> i64 {
        Game::BOARD_ROW as i64 - 1
    }

    /// should only be called at cleanup
    fn clear(&mut self) {
        self.0 = vec![];
    }
}

fn update_cell_state(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut field: ResMut<CellField>,
    mut ev_update_cell: EventReader<UpdateCellEvent>,
) {
    for e in ev_update_cell.read() {
        let cell = field.get_cell_mut(e.coord);
        cell.state = e.new_state;
        cell.draw(&mut cmds, &mut meshes, &mut materials);

        // TODO: doing some of the field_entity's job
        #[cfg(debug_assertions)]
        {
            use bevy_rapier2d::prelude::*;
            if !cell.is_passable() {
                cmds.spawn(RigidBody::Fixed)
                    .insert(Collider::cuboid(
                        Game::CELL_SIZE / 2.0,
                        Game::CELL_SIZE / 2.0,
                    ))
                    .insert(TransformBundle::from(Transform::from_xyz(
                        cell.coord.0 as f32 * Game::CELL_SIZE,
                        cell.coord.1 as f32 * Game::CELL_SIZE,
                        super::layer::Layer::Debug.into(),
                    )));
            }
        }
    }
}

use super::unit_move::DebugTargetCell;
/// use left&right shift to change a cell to empty or iron
#[cfg(debug_assertions)]
fn debug_alter_cell_state(
    kb_input: Res<ButtonInput<KeyCode>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_tf: Query<(&Camera, &GlobalTransform)>,
    field: Res<CellField>,
    mut ev_update_cell: EventWriter<UpdateCellEvent>,
    mut ev_debug_set_unit_dest: EventWriter<DebugSetUnitDestEvent>,
    target: ResMut<DebugTargetCell>,
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
    if cell_coord.0 < CellField::min_col()
        || cell_coord.0 > CellField::max_col()
        || cell_coord.1 < CellField::min_row()
        || cell_coord.1 > CellField::max_row()
    {
        return;
    }
    if target_state.unwrap() == field.get_cell(cell_coord).state {
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
    if let Some(t) = target.0 {
        ev_debug_set_unit_dest.send(DebugSetUnitDestEvent(t));
    }
}

/// calculate all cell's distance to the debug target cell using BFS
use super::input_event::DebugSetUnitDestEvent;
#[cfg(debug_assertions)]
fn debug_calculate_distance(
    mut field: ResMut<CellField>,
    mut ev_debug_set_unit_dest: EventReader<DebugSetUnitDestEvent>,
) {
    if ev_debug_set_unit_dest.is_empty() {
        return;
    }
    let dest = ev_debug_set_unit_dest.read().next().unwrap().0;

    // calculate distance of each cell to the dest
    info!("re-calculating distances");
    for c in 0..field.0.len() as i64 {
        for r in 0..field.0[0].len() as i64 {
            field.get_cell_mut((c, r)).debug_distance = f32::MAX;
        }
    }
    field.get_cell_mut(dest).debug_distance = 0.0;

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
            if !field.get_cell((nx, ny)).is_passable() {
                continue;
            }
            // avoid moving diagonally through two obstacles
            if dx[i] * dy[i] != 0 {
                if !field.get_cell((x, ny)).is_passable()
                    && !field.get_cell((nx, y)).is_passable()
                {
                    continue;
                }
            }

            let new_distance =
                distance + ((dx[i] * dx[i] + dy[i] * dy[i]) as f32).sqrt();
            let ncell = &mut field.get_cell_mut((nx, ny));
            if new_distance < ncell.debug_distance {
                ncell.debug_distance = new_distance;
                q.push_back((nx, ny, new_distance));
            }
        }
    }
}

#[cfg(debug_assertions)]
fn debug_calculate_direction(
    mut field: ResMut<CellField>,
    mut ev_debug_set_unit_dest: EventReader<DebugSetUnitDestEvent>,
    mut ev_debug_redraw: EventWriter<DebugRedrawEvent>,
) {
    if ev_debug_set_unit_dest.is_empty() {
        return;
    }
    let dest = ev_debug_set_unit_dest.read().next().unwrap().0;

    // calculate direction of each passable cell
    info!("re-calculating directions");
    for c in CellField::min_col()..=CellField::max_col() {
        for r in CellField::min_row()..=CellField::max_row() {
            let cell = field.get_cell((c, r));
            if !cell.is_passable() || cell.coord == dest {
                field.get_cell_mut((c, r)).debug_direction = Vec2::NAN;
                continue;
            }
            let reachable_adj_cells: Vec<&Cell> =
                field.get_reachable_adjacent_cells(cell);
            let reachable_adj_cells = {
                let mut ret = vec![];
                for c in reachable_adj_cells {
                    if c.debug_distance != f32::MAX {
                        ret.push(c);
                    }
                }
                ret
            };

            let mut to_adj_vecs = vec![];
            let min_adj_dist = reachable_adj_cells
                .iter()
                .fold(f32::MAX, |accu, &x| accu.min(x.debug_distance));

            if reachable_adj_cells.len() < 8 {
                // this cell is adjacent to some obstacle, just make it point to
                // the cell with min distance that is reachable
                to_adj_vecs = vec![];
                for target in &reachable_adj_cells {
                    if !target.is_passable() {
                        continue;
                    }
                    let (tar_c, tar_r) = target.coord;
                    if (tar_c - c) * (tar_r - r) != 0
                        && !field.get_cell((tar_c, r)).is_passable()
                        && !field.get_cell((c, tar_r)).is_passable()
                    {
                        continue;
                    }
                    if target.debug_distance - min_adj_dist <= f32::EPSILON {
                        to_adj_vecs = vec![Vec2 {
                            x: (target.coord.0 - c) as f32,
                            y: (target.coord.1 - r) as f32,
                        }];
                        break;
                    }
                }
            } else {
                for ac in &reachable_adj_cells {
                    // this adjacent cell is just the dest
                    if ac.coord == dest {
                        let dir = Vec2 {
                            x: (ac.coord.0 - c) as f32,
                            y: (ac.coord.1 - r) as f32,
                        };
                        to_adj_vecs = vec![dir];
                        break;
                    }

                    let dir_ac = Vec2 {
                        x: (ac.coord.0 - cell.coord.0) as f32,
                        y: (ac.coord.1 - cell.coord.1) as f32,
                    }
                    .normalize()
                        * (min_adj_dist / ac.debug_distance);
                    to_adj_vecs.push(dir_ac);
                }
            }
            let direction = to_adj_vecs.iter().sum::<Vec2>().normalize();
            // info!("direction for {:?} is {:?}", cell.coord, direction);

            field.get_cell_mut((c, r)).debug_direction = direction;
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
