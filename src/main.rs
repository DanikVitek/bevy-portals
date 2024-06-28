use bevy::prelude::*;
use bevy_editor_pls::EditorPlugin;
use bevy_portals::{
    resource::{Controls, ControlsConfig},
    system::{input::*, player::*, setup::*},
};
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(EditorPlugin::default())
        .init_resource::<ControlsConfig>()
        .init_resource::<Controls>()
        .add_systems(Startup, (setup_player, setup_physics, cursor_grab))
        .add_systems(Update, ((input_mappings, movement).chain(), rotation))
        .run();
}
