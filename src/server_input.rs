use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use lightyear::prelude::*;
use server::InputEvent;

use crate::{
    ClientIds,
    protocol::{Inputs, PlayerPosition},
    world::LastTouchedTime,
};

pub struct ServerInputPlugin;

impl Plugin for ServerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, movement);
        app.add_systems(PostUpdate, sync_positions);
    }
}

fn movement(
    mut position_query: Query<(&mut Velocity, &mut ExternalForce, &LastTouchedTime)>,
    mut input_reader: EventReader<InputEvent<Inputs>>,
    client_ids: Res<ClientIds>,
    time: Res<Time>,
) {
    for input in input_reader.read() {
        let client_id = input.from();
        if let Some(input) = input.input() {
            let client_ids = client_ids.0.read().unwrap();
            if let Some(player_entity) = client_ids.get(&client_id.to_bits()) {
                if let Ok((velocity, force, last_touched)) = position_query.get_mut(*player_entity)
                {
                    if time.elapsed_secs() - last_touched.0 < 1.0 || last_touched.1 {
                        shared_movement_behaviour(velocity, force, input);
                    }
                }
            }
        }
    }
}

fn shared_movement_behaviour(
    mut velocity: Mut<Velocity>,
    mut force: Mut<ExternalForce>,
    input: &Inputs,
) {
    let lin = velocity.linvel.normalize();
    let multiplier = 0.1f32;
    let up = Vec3::Y;

    match input {
        Inputs::Direction(direction) => {
            if direction.forward {
                let forward_torque = up.cross(lin).normalize();
                force.torque += forward_torque * multiplier;
                force.torque = force.torque.clamp_length(-10.0, 10.0);
            }
            if direction.back {
                let forward_torque = up.cross(lin).normalize();
                force.torque += -forward_torque * multiplier;
                force.torque = force.torque.clamp_length(-10.0, 10.0);
            }
            if direction.left {
                let rotated = up.cross(Quat::from_rotation_y(PI * 0.5) * lin);
                force.torque += rotated * multiplier * 2.0;
                force.torque = force.torque.clamp_length(-10.0, 10.0);
            }
            if direction.right {
                let rotated = up.cross(Quat::from_rotation_y(PI * 0.5) * lin);
                force.torque += -rotated * multiplier * 2.0;
                force.torque = force.torque.clamp_length(-10.0, 10.0);
            }
            if direction.reset {
                velocity.angvel = Vec3::ZERO;
            }
        }
        _ => force.torque = Vec3::ZERO,
    }
}

fn sync_positions(mut players: Query<(&mut PlayerPosition, &Transform)>) {
    for (mut position, transform) in players.iter_mut() {
        *position = PlayerPosition(transform.translation, transform.rotation);
    }
}
