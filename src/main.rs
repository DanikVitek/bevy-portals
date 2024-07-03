use bevy::{prelude::*, window::WindowResolution};
#[cfg(feature = "debug")]
use bevy_editor_pls::EditorPlugin;
use bevy_gltf_components::ComponentsFromGltfPlugin;
use bevy_portals::{
    domain::{debug_info, input, player, portal, scene, ui::{self, CrosshairMaterial}, AppExt},
    resource::{Controls, ControlsConfig, MouseSensitivity},
};
use bevy_rapier3d::prelude::*;
use bevy_registry_export::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            #[cfg(feature = "debug")]
            RapierDebugRenderPlugin::default().disabled(),
            // #[cfg(feature = "debug")]
            // EditorPlugin::default().in_new_window(Window {
            //     resolution: WindowResolution::new(1920.0, 1080.0),
            //     position: WindowPosition::Centered(MonitorSelection::Index(1)),
            //     decorations: true,
            //     ..Default::default()
            // }),
            ExportRegistryPlugin::default(),
            ComponentsFromGltfPlugin::default(),
        ))
        .register_types() // domain::AppExt
        .init_resource::<ControlsConfig>()
        .init_resource::<MouseSensitivity>()
        .init_resource::<Controls>()
        .add_plugins(UiMaterialPlugin::<CrosshairMaterial>::default())
        .add_systems(
            Startup,
            (
                player::setup,
                scene::setup,
                input::setup,
                debug_info::setup,
                ui::setup,
            ),
        )
        .add_systems(
            Update,
            (
                (
                    input::input_mappings,
                    (
                        (player::movement, debug_info::player_is_grounded).chain(),
                        portal::shoot_portal,
                    ),
                )
                    .chain(),
                player::rotation,
                input::cursor_grab,
                input::cursor_ungrab,
                debug_info::portal_surface_gizmo,
                debug_info::portal_gizmo,
            ),
        )
        .run();
}
