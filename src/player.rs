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
            .add_systems(Update, (watch_mouse, squish_player).chain())
            .add_systems(Update, camera_follow);
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

fn squish_player(
    query: Query<(&LinearVelocity, &mut Transform), With<Player>>
) {
    for (linear_velocity, mut transform) in query {
        let squish = (linear_velocity.xy()/500.).clamp(Vec2::ZERO, Vec2::ONE);// * linear_velocity.length().min(1.);

        transform.rotation = Quat::from_euler(EulerRot::XYZ, squish.y, squish.x, 0.);
    }
}