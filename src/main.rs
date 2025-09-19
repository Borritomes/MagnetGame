#![allow(unused_parens)]
pub mod gun;
pub mod item;
pub mod magnet;
pub mod player;

use std::env;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use gun::*;
use magnet::*;
use player::*;

fn main() {
    unsafe {env::set_var("RUST_BACKTRACE", "1");}

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins((PlayerPlugin, GunPlugin, MagnetPlugin))
        .run();
}
