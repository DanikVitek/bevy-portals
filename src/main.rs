use bevy::prelude::*;
use bevy_editor_pls::EditorPlugin;
use bevy_portals::{
    domain::{debug_info, input, player, scene},
    resource::{Controls, ControlsConfig, MouseSensitivity},
};
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(EditorPlugin::default())
        .init_resource::<ControlsConfig>()
        .init_resource::<MouseSensitivity>()
        .init_resource::<Controls>()
        .add_systems(
            Startup,
            (player::setup, scene::setup, input::setup, debug_info::setup),
        )
        .add_systems(
            Update,
            (
                (input::input_mappings, player::movement).chain(),
                player::rotation,
                debug_info::player_is_grounded,
                input::cursor_grab,
                input::cursor_ungrab,
            ),
        )
        .run();
}
