use crate::gun::*;
use crate::item::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(FixedUpdate, player_movement)
            .add_systems(Update, (camera_follow, watch_mouse));
    }
}

#[derive(PhysicsLayer, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Player,
    Walls,
    Magnet,
    Projectiles,
    Bullets,
    MagnetPassthrough,
}

pub struct CollisionGroup;

impl CollisionGroup {
    pub fn player() -> CollisionLayers {
        CollisionLayers::new(
            GameLayer::Player,
            [
                GameLayer::Default,
                GameLayer::Walls,
                GameLayer::Magnet,
                GameLayer::MagnetPassthrough,
            ],
        )
    }
    pub fn magnet() -> CollisionLayers {
        CollisionLayers::new(
            [GameLayer::Magnet, GameLayer::Projectiles],
            [
                GameLayer::Default,
                GameLayer::Walls,
                GameLayer::Projectiles,
                GameLayer::Player,
                GameLayer::Bullets,
            ],
        )
    }
    pub fn bullet() -> CollisionLayers {
        CollisionLayers::new(
            [GameLayer::Bullets, GameLayer::Projectiles],
            [
                GameLayer::Default,
                GameLayer::Walls,
                GameLayer::Projectiles,
                GameLayer::MagnetPassthrough,
            ],
        )
    }
    pub fn magnet_passthrough() -> CollisionLayers {
        CollisionLayers::new(
            GameLayer::MagnetPassthrough,
            [
                GameLayer::Default,
                GameLayer::Walls,
                GameLayer::Projectiles,
                GameLayer::Player,
                GameLayer::Bullets,
            ],
        )
    }
    pub fn walls() -> CollisionLayers {
        CollisionLayers::new(
            GameLayer::Walls,
            [
                GameLayer::Default,
                GameLayer::Walls,
                GameLayer::Projectiles,
                GameLayer::Player,
                GameLayer::Bullets,
                GameLayer::Magnet,
            ],
        )
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct LookAtCursor;

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let owl = asset_server.load("owl.jpg");
    let spider = asset_server.load("spider.png");

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
            Name::new("Player"),
            Player,
            LookAtCursor,
            Position::from_xy(0., 0.),
            LinearVelocity::default(),
            LockedAxes::ROTATION_LOCKED,
            RigidBody::Dynamic,
            Collider::circle(32.),
            Sprite {
                image: spider,

                custom_size: Some(Vec2::new(64., 64.)),
                ..default()
            },
            CollisionGroup::player(),
        ))
        .id();
    //make gun
    commands.spawn((
        Name::new("Gun"),
        Sprite::from_color(Color::srgb(0.3, 0.4, 0.7), Vec2::new(16., 16.)),
        ShootProjectiles,
        Item,
        ShootCooldown::new(0.1),
        ProjectileSpeed(1000.),
        ProjectileType::Bullet,
        ActivationKeyCode(KeyCode::Space),
        ChildOf(player),
        Transform {
            translation: Vec3::new(0., 32., 1.),
            ..default()
        },
        Equipped,
    ));
    //make weak gun
    commands.spawn((
        Name::new("WeakGun"),
        Sprite::from_color(Color::srgb(0.4, 1.0, 0.2), Vec2::new(16., 16.)),
        ShootProjectiles,
        Item,
        ShootCooldown::new(0.05),
        ProjectileSpeed(2000.),
        ProjectileType::WeakBullet,
        ActivationKeyCode(KeyCode::KeyC),
        ChildOf(player),
        Transform {
            translation: Vec3::new(0., 32., 1.),
            ..default()
        },
        Equipped,
    ));
    //make magnet
    commands.spawn((
        Name::new("Magnet"),
        Sprite::from_color(Color::srgb(0., 0., 0.95), Vec2::new(16., 16.)),
        ShootProjectiles,
        Item,
        ProjectileSpeed(500.),
        ProjectileType::Magnet,
        ProjectileFriction(0.05),
        ActivationKeyCode(KeyCode::ShiftLeft),
        ChildOf(player),
        Transform {
            translation: Vec3::new(0., 32., 1.),
            ..default()
        },
        Equipped,
    ));
    //magnet passthrough
    commands.spawn((
        Sprite::from_color(Color::srgb(0.25, 0.25, 1.), Vec2::new(128., 16.)),
        Collider::rectangle(128., 16.),
        Position::from_xy(0., 128.),
        RigidBody::Static,
        CollisionGroup::magnet_passthrough(),
    ));
    //wall
    commands.spawn((
        Sprite::from_color(Color::srgb(0.4, 0.4, 1.), Vec2::new(128., 16.)),
        Collider::rectangle(128., 16.),
        Position::from_xy(0., -128.),
        RigidBody::Static,
        CollisionGroup::walls(),
    ));
}

const SPEED: f32 = 320.;

fn watch_mouse(
    window: Single<&Window, With<PrimaryWindow>>,
    query_camera: Query<(&Camera, &GlobalTransform), Without<Item>>,
    query: Query<&mut Transform, With<LookAtCursor>>,
) {
    let (camera, camera_transform) = query_camera.single().unwrap();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        for mut transform in query {
            let direction = world_position.xy() - transform.translation.xy();
            let angle = direction.y.atan2(direction.x) - 1.57079633;
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<&mut LinearVelocity, With<Player>>,
) {
    let mut wish_dir = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        wish_dir += Vec2::new(0., 1.)
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        wish_dir += Vec2::new(0., -1.)
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        wish_dir += Vec2::new(1., 0.)
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        wish_dir += Vec2::new(-1., 0.)
    }

    wish_dir = wish_dir.normalize_or_zero();
    let wish_speed = (wish_dir * SPEED).length();
    for mut linear_velocity in query {
        let current_speed = linear_velocity.dot(wish_dir);
        let add_speed = wish_speed - current_speed;
        if add_speed != 0. {
            let mut accel_speed = 10. * time.delta_secs() * wish_speed;
            if accel_speed > add_speed {
                accel_speed = add_speed
            }

            let new_speed = wish_dir * accel_speed;
            linear_velocity.x += new_speed.x;
            linear_velocity.y += new_speed.y;
        }

        let speed = ((linear_velocity.x * linear_velocity.x)
            + (linear_velocity.y * linear_velocity.y))
            .sqrt();

        let control = if speed < 100. { 100. } else { speed };
        let mut new_speed = speed - (time.delta_secs() * control * 4.);

        if new_speed < 0. {
            new_speed = 0.
        }

        new_speed /= speed;
        if new_speed.is_finite() == false {
            new_speed = 0.
        }

        linear_velocity.x *= new_speed;
        linear_velocity.y *= new_speed;
    }
}

fn camera_follow(
    mut camera_transform: Single<&mut Transform, (With<Camera>, Without<Player>)>,
    player_transform: Single<&Transform, (With<Player>, Without<Camera>)>,
) {
    camera_transform.translation = player_transform.translation
}

/*fn squish_player(query: Query<(&LinearVelocity, &mut Transform), With<Player>>) {
    for (linear_velocity, mut transform) in query {
        let move_dir = (linear_velocity.xy()).clamp(-Vec2::ONE, Vec2::ONE);
        let squish = (move_dir / 640.) * linear_velocity.length();

        let is_both_negative = (squish.x < 0. && squish.y < 0.);

        if is_both_negative == true {
            transform.rotation = Quat::from_euler(EulerRot::XYZ, squish.y, squish.x, 0.);
        } else {
            transform.rotation = Quat::from_euler(EulerRot::XYZ, squish.y, -squish.x, 0.);
        }
    }
}
*/