use bevy::prelude::*;
use bevy_editor_pls::EditorPlugin;
use bevy_portals::{
    resource::{Controls, ControlsConfig, MouseSensitivity},
    system::{debug_info, input, player, setup},
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
            (
                setup::player,
                setup::scene,
                setup::cursor_grab,
                setup::debug_info,
            ),
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
