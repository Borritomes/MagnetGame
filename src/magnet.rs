use crate::gun::*;
use avian2d::prelude::*;
use bevy::prelude::*;

pub struct MagnetPlugin;

impl Plugin for MagnetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (magnet_friction, attract_bullets, age_magnet));
    }
}

#[derive(Component)]
#[require(ProjectileSpeed, ProjectileFriction(0.1))]
pub struct Magnet;

#[derive(Component)]
pub struct MagnetStrength(pub f32);

#[derive(Component)]
pub struct MagentAliveTime {
    pub max: f32,
    pub current: f32,
}

impl MagentAliveTime {
    pub fn new(max: f32) -> MagentAliveTime {
        MagentAliveTime {
            max: max,
            current: 0.,
        }
    }
}

fn magnet_friction(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &mut LinearVelocity,
            &ProjectileFriction,
            &mut RigidBody,
        ),
        With<Magnet>,
    >,
) {
    for (entity, mut linear_velocity, magnet_friction, mut rigid_body) in query {
        if linear_velocity.xy().length() < 10. {
            commands.entity(entity).remove::<MagentAliveTime>();
            *rigid_body = RigidBody::Static;
            continue;
        }
        linear_velocity.x *= 1.0 - magnet_friction.0;
        linear_velocity.y *= 1.0 - magnet_friction.0;
    }
}

fn attract_bullets(
    time: Res<Time>,
    query: Query<(&mut LinearVelocity, &Transform), With<Bullet>>,
    query_magnet: Single<(&Transform, &MagnetStrength), With<Magnet>>,
) {
    let magnet_transform = query_magnet.0;
    let magnet_strength = query_magnet.1;

    for (mut linear_velocity, transform) in query {
        let prediction =
            transform.translation.xy() + (linear_velocity.xy() * (time.delta_secs() * 2.0));

        let delta = magnet_transform.translation.xy() - prediction;
        linear_velocity.x += (delta.x * magnet_strength.0);
        linear_velocity.y += (delta.y * magnet_strength.0);
    }
}

fn age_magnet(
    time: Res<Time>,
    query: Query<(&mut MagentAliveTime, &mut ProjectileFriction), With<Magnet>>,
) {
    for (mut magnet_alive_time, mut magnet_friction) in query {
        magnet_alive_time.current += time.delta_secs();
        if magnet_alive_time.current > magnet_alive_time.max {
            magnet_friction.0 += 0.0125
        }
    }
}
