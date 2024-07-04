use super::Game;
use super::Layer;
use bevy::tasks::futures_lite::io::Empty;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

/// Cell is **irrelevant** to any entity(except its visual), it just provides
/// information to render the color of the grid, and assist in path finding
/// algorithm.
/// if there is entity(e.g. building, mine) occupying this cell, it just convert
/// itself to matching movable state, it does not manager the entity.
pub struct Cell {
    pub coord: (i64, i64),
    pub state: CellState,
}

#[derive(Default)]
pub enum CellState {
    #[default]
    Empty, // all else are actually implemented as entities
    Water,
    Iron,
    Rock,
    Building,
    Mine,
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

    /// draws the Cell's visual entity
    pub fn draw(
        &self,
        coord: (i64, i64),
        cmds: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) {
        let (r, c) = coord;
        // draw a bigger outer square as boarder
        let shape_outer = Mesh2dHandle(
            meshes.add(Rectangle::new(Game::CELL_SIZE, Game::CELL_SIZE)),
        );
        cmds.spawn(MaterialMesh2dBundle {
            mesh: shape_outer,
            material: materials.add(Color::AZURE),
            transform: Transform::from_xyz(
                c as f32 * Game::CELL_SIZE,
                r as f32 * Game::CELL_SIZE,
                Layer::GameMap.into(),
            ),
            ..default()
        });

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
            _ => todo!(),
        };
        let mut inner_z: f32 = Layer::GameMap.into();
        inner_z += 0.1;
        cmds.spawn(MaterialMesh2dBundle {
            mesh: shape_inner,
            material: materials.add(color),
            transform: Transform::from_xyz(
                c as f32 * Game::CELL_SIZE,
                r as f32 * Game::CELL_SIZE,
                inner_z,
            ),
            ..default()
        });
    }
}
