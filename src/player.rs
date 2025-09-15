use crate::gun::*;
use crate::item::*;
use avian2d::prelude::*;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(FixedUpdate, player_movement)
            .add_systems(Update, camera_follow);
    }
}

#[derive(Component)]
pub struct Player;

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let owl = asset_server.load("owl.jpg");

    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_image(owl),
        Transform {
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
    ));
    let player = commands
        .spawn((
            Player,
            Position::from_xy(0., 0.),
            LinearVelocity::default(),
            LockedAxes::ROTATION_LOCKED,
            RigidBody::Dynamic,
            Collider::rectangle(32., 32.),
            Sprite::from_color(Color::srgb(1., 0.1, 0.25), Vec2::new(32., 32.)),
        ))
        .id();
    //make gun
    commands.spawn((
        ShootProjectiles,
        Item,
        ShootCooldown::new(0.1),
        ProjectileSpeed(1000.),
        ProjectileType::Bullet,
        ActivationKeyCode(KeyCode::Space),
        ChildOf(player),
        Transform::default(),
        Equipped(true),
    ));
    //make magnet
    commands.spawn((
        ShootProjectiles,
        Item,
        ProjectileSpeed(500.),
        ProjectileType::Magnet,
        ProjectileFriction(0.05),
        ActivationKeyCode(KeyCode::ShiftLeft),
        ChildOf(player),
        Transform::default(),
        Equipped(true),
    ));
}

const SPEED: f32 = 320.;

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<&mut LinearVelocity, With<Player>>,
) {
    let mut wish_dir = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        wish_dir += Vec3::new(0., 1., 0.)
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        wish_dir += Vec3::new(0., -1., 0.)
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        wish_dir += Vec3::new(1., 0., 0.)
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        wish_dir += Vec3::new(-1., 0., 0.)
    }

    wish_dir = wish_dir.normalize_or_zero();
    for mut linear_velocity in query {
        linear_velocity.x = wish_dir.x * SPEED;
        linear_velocity.y = wish_dir.y * SPEED;
    }
}

fn camera_follow(
    mut camera_transform: Single<&mut Transform, (With<Camera>, Without<Player>)>,
    player_transform: Single<&Transform, (With<Player>, Without<Camera>)>,
) {
    camera_transform.translation = player_transform.translation
}
