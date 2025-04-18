use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use lightyear::prelude::client::Predicted;

use crate::{
    protocol::{PlayerColor, PlayerPosition},
    world::{GravityModifier, LastTouchedId, LastTouchedTime},
};

pub struct PlayerPlugin {
    pub physics: bool,
    pub player_count: u8,
}

#[derive(Resource, Debug)]
struct Physics(bool);

#[derive(Resource)]
pub struct SpawnedPlayersCount {
    pub current: u8,
    pub max: u8,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_player_model);
        app.insert_resource(SpawnedPlayersCount {
            current: 0,
            max: self.player_count,
        });
        app.insert_resource(Physics(self.physics));
    }
}

fn attach_player_model(
    player_query: Query<
        (&PlayerPosition, &PlayerColor, Entity),
        (Without<Transform>, Without<Predicted>),
    >,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_count: Res<SpawnedPlayersCount>,
    physics: Res<Physics>,
) {
    if player_count.current == player_count.max {
        let mut c = 0;
        for (position, color, entity) in player_query.iter() {
            c += 1;
            info!(position=?position, phys=?physics,"attach player model");
            commands.get_entity(entity).unwrap().insert((
                Mesh3d(meshes.add(Sphere::new(0.5))),
                MeshMaterial3d(materials.add(color.0)),
                Transform::from_translation(position.0),
            ));
            commands.get_entity(entity).unwrap().log_components();
            if physics.0 {
                commands
                    .get_entity(entity)
                    .unwrap()
                    .insert(Collider::ball(0.5))
                    .insert(Restitution::coefficient(0.7))
                    .insert(RigidBody::Dynamic)
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(Velocity {
                        linvel: Vec3::new(0.0, 0.0, 0.0),
                        angvel: Vec3::new(0.0, 0.0, 0.0),
                    })
                    .insert(ExternalForce {
                        force: Vec3::ZERO,
                        torque: Vec3::ZERO,
                    })
                    .insert(GravityScale(1.0))
                    .insert(Ccd::enabled())
                    .insert(LastTouchedId(0))
                    .insert(LastTouchedTime(0.0, false))
                    .insert(GravityModifier {
                        base_gravity: 1.0,
                        remaining: Timer::from_seconds(0.0, TimerMode::Once),
                        current: 1.0,
                    });
            }
        }
        if c != 0 {
            dbg!(c);
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub position: PlayerPosition,
    pub color: PlayerColor,
}
