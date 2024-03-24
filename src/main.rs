use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};

fn main() {
    App::new().add_plugins((DefaultPlugins, GamePlugin)).run();
}

#[derive(Component)]
struct Ball;

#[derive(Resource, Default)]
struct WorldCoords(Vec2);

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldCoords(Vec2 { x: 0., y: 0. }))
            .add_systems(Startup, setup)
            .add_systems(Update, change_position);
    }
}

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    cmds.spawn(Camera2dBundle::default());

    let shape = Mesh2dHandle(meshes.add(Circle { radius: 30.0 }));
    let color = Color::hsl(360.0, 0.95, 0.7);

    cmds.spawn((
        MaterialMesh2dBundle {
            mesh: shape,
            material: materials.add(color),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Ball,
    ));
}

fn change_position(
    mut query: Query<&mut Transform, With<Ball>>,
    mb: Res<ButtonInput<MouseButton>>,
    mut cm: EventReader<CursorMoved>,

    mut coords: ResMut<WorldCoords>,
    w: Query<&Window, With<PrimaryWindow>>,
    c: Query<(&Camera, &GlobalTransform)>,
) {
    let (cam, cam_transform) = c.single();
    let window = w.single();

    if mb.just_pressed(MouseButton::Left) {
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| cam.viewport_to_world(cam_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            coords.0 = world_position;
            info!("coords={}", world_position);
        }
        for e in cm.read() {
            info!("click={}", e.position);
            for mut transform in &mut query {
                transform.translation.x = coords.0.x;
                transform.translation.y = coords.0.y;
            }
        }
    }
}
