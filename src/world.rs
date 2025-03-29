use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{track_gen::Track, track_mesh::generate_mesh};

pub struct WorldPlugin {
    pub physics: bool,
}

#[derive(Resource)]
struct Physics(bool);

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_world);
        app.insert_resource(Physics(self.physics));
    }
}

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    physics: Res<Physics>,
) {
    let mut e = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(100.0, 0.1, 100.0))),
        MeshMaterial3d(materials.add(Color::oklab(0.74, -0.12, 0.11))),
    ));
    e.insert(Transform::from_xyz(0.0, -2.0, 0.0));
    if physics.0 {
        e.insert(Collider::cuboid(100.0, 0.1, 100.0));
    }
    let track = Track::generate(1234, 30.0);
    for segment in track.segments {
        commands.spawn((
            Mesh3d(meshes.add(generate_mesh(&segment.block_type))),
            MeshMaterial3d(materials.add(Color::oklab(0.8, -0.12, 0.11))),
            Transform::IDENTITY
                .with_translation(segment.transform.position)
                .with_rotation(segment.transform.rotation),
        ));
    }
}
