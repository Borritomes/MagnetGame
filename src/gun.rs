use crate::item::*;
use crate::magnet::*;
use crate::player::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct GunPlugin;

impl Plugin for GunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (clean_projectiles, shoot_projectiles));
    }
}

#[derive(Component)]
pub struct ProjectileFriction(pub f32);

#[derive(Component)]
pub struct ShootCooldown {
    pub remaining: f32,
    pub cooldown: f32,
}

impl ShootCooldown {
    pub fn new(cooldown: f32) -> Self {
        Self {
            remaining: cooldown,
            cooldown,
        }
    }
}

impl Default for ShootCooldown {
    fn default() -> Self {
        ShootCooldown::new(0.0)
    }
}

#[derive(Component)]
pub struct ProjectileLifetime {
    pub remaining: f32,
    pub lifetime: f32,
}

impl ProjectileLifetime {
    fn new(lifetime: f32) -> Self {
        ProjectileLifetime {
            remaining: lifetime,
            lifetime: lifetime,
        }
    }
}

#[derive(Component)]
#[require(ShootCooldown, ProjectileFriction(0.0))]
pub struct ShootProjectiles;

#[derive(Component)]
pub struct ProjectileSpeed(pub f32);

#[derive(Component)]
pub struct ActivationKeyCode(pub KeyCode);

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub enum ProjectileType {
    Magnet,
    Bullet,
    WeakBullet,
}

impl Default for ProjectileSpeed {
    fn default() -> Self {
        ProjectileSpeed(100.)
    }
}

#[derive(Component)]
#[require(LockedAxes::ROTATION_LOCKED, GravityScale(0.0))]
struct Projectile;

fn clean_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &mut ProjectileLifetime)>,
) {
    for (entity, mut projectile_lifetime) in query {
        projectile_lifetime.remaining -= time.delta_secs();
        if projectile_lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn shoot_projectiles(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    query_camera: Query<(&Camera, &GlobalTransform), Without<Item>>,
    query: Query<
        (
            &GlobalTransform,
            &mut ShootCooldown,
            &ProjectileSpeed,
            &ActivationKeyCode,
            &ProjectileType,
            &ProjectileFriction,
        ),
        (With<Item>, With<ShootProjectiles>, With<Equipped>),
    >,
    query_magnets: Query<Entity, With<Magnet>>,
) {
    let (camera, camera_transform) = query_camera.single().unwrap();
    
    for (
        transform,
        mut shoot_cooldown,
        projectile_speed,
        activation_key_code,
        projectile_type,
        projectile_friction,
    ) in query
    {
        if shoot_cooldown.remaining - time.delta_secs() > 0.0 {
            shoot_cooldown.remaining -= time.delta_secs();
            continue;
        }
        
        if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
        {
            let gun_position = transform.translation().xy();
            let delta = world_position - gun_position;
            let direction = delta.normalize_or_zero();
            
            if keyboard_input.just_pressed(activation_key_code.0) {
                shoot_cooldown.remaining = shoot_cooldown.cooldown;
                /*
                Position::from_xy(transform.translation().x, transform.translation().y),
                LinearVelocity(direction * projectile_speed.0),
                */
                match projectile_type {
                    ProjectileType::Magnet => {
                        for entity in query_magnets {
                            commands.entity(entity).despawn();
                        }
                        commands.spawn((
                            ProjectileFriction(projectile_friction.0),
                            Collider::rectangle(16., 16.),
                            Sprite::from_color(Color::srgb(0.9, 0.3, 0.3), Vec2::new(16., 16.)),
                            Magnet,
                            MagentAliveTime::new(3.),
                            Projectile,
                            Position::from_xy(
                                transform.translation().x + direction.x * 26.,
                                transform.translation().y + direction.y * 26.,
                            ),
                            LinearVelocity(direction * projectile_speed.0),
                            MagnetStrength(0.9),
                            CollisionGroup::magnet(),
                            RigidBody::Dynamic,
                        ));
                    }
                    ProjectileType::Bullet => {
                        commands.spawn((
                            ProjectileAttraction(2.),
                            ProjectileLifetime::new(8.),
                            ProjectileFriction(projectile_friction.0),
                            Collider::rectangle(8., 8.),
                            Sprite::from_color(Color::srgb(1., 1., 1.), Vec2::new(8., 8.)),
                            Bullet,
                            Projectile,
                            Position::from_xy(
                                transform.translation().x + direction.x * 22.,
                                transform.translation().y + direction.y * 22.,
                            ),
                            LinearVelocity(direction * projectile_speed.0),
                            CollisionGroup::bullet(),
                            RigidBody::Dynamic,
                        ));
                    }
                    ProjectileType::WeakBullet => {
                        commands
                            .spawn((
                                ProjectileAttraction(6.),
                                ProjectileLifetime::new(5.),
                                ProjectileFriction(projectile_friction.0),
                                Collider::rectangle(8., 8.),
                                Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::new(8., 8.)),
                                Bullet,
                                Projectile,
                                Position::from_xy(
                                    transform.translation().x + direction.x * 22.,
                                    transform.translation().y + direction.y * 22.,
                                ),
                                LinearVelocity(direction * projectile_speed.0),
                                CollisionGroup::bullet(),
                                RigidBody::Dynamic,
                                CollisionEventsEnabled,
                            ))
                            .observe(
                                |trigger: Trigger<OnCollisionStart>, mut commands: Commands| {
                                    commands.entity(trigger.target()).despawn();
                                },
                            );
                    }
                }
            }
        }
    }
}
