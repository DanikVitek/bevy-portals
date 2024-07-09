use bevy::{prelude::*, render::camera::CameraProjectionPlugin};
#[cfg(feature = "debug")]
use bevy::window::WindowResolution;
#[cfg(feature = "debug")]
use bevy_editor_pls::EditorPlugin;
// use bevy_gltf_components::ComponentsFromGltfPlugin;
use bevy_portals::{
    domain::{
        debug_info, input, player,
        portal::{self, Portal1, Portal2, PortalPerspectiveProjection, PortalViewMaterial, SpawnPortal},
        scene,
        ui::{self, CrosshairMaterial},
        AppExt,
    },
    resource::{Controls, ControlsConfig, Fov, MouseSensitivity},
};
use bevy_rapier3d::prelude::*;
// use bevy_registry_export::*;
use bevy_tnua::{controller::TnuaControllerPlugin, TnuaUserControlsSystemSet};
use bevy_tnua_rapier3d::TnuaRapier3dPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            #[cfg(feature = "debug")]
            RapierDebugRenderPlugin::default().disabled(),
            #[cfg(feature = "debug")]
            EditorPlugin::default().in_new_window(Window {
                resolution: WindowResolution::new(1920.0, 1080.0),
                position: WindowPosition::Centered(MonitorSelection::Index(1)),
                decorations: true,
                ..Default::default()
            }),
            // ExportRegistryPlugin::default(),
            // ComponentsFromGltfPlugin::default(),
            TnuaControllerPlugin::default(),
            TnuaRapier3dPlugin::default(),
        ))
        .register_types() // domain::AppExt
        .init_resource::<ControlsConfig>()
        .init_resource::<MouseSensitivity>()
        .init_resource::<Fov>()
        .init_resource::<Controls>()
        .add_event::<SpawnPortal<Portal1>>()
        .add_event::<SpawnPortal<Portal2>>()
        .add_plugins((
            UiMaterialPlugin::<CrosshairMaterial>::default(),
            MaterialPlugin::<PortalViewMaterial>::default(),
            CameraProjectionPlugin::<PortalPerspectiveProjection>::default(),
        ))
        .add_systems(
            Startup,
            (
                player::setup.pipe(ui::setup),
                scene::setup,
                input::setup,
                debug_info::setup,
            ),
        )
        .add_systems(
            Update,
            (
                (
                    input::input_mappings,
                    (
                        (
                            player::movement.in_set(TnuaUserControlsSystemSet),
                            debug_info::player_is_grounded,
                        )
                            .chain(),
                        (
                            portal::shoot_portal,
                            (
                                portal::spawn_portal::<Portal1>,
                                portal::spawn_portal::<Portal2>,
                            ),
                        )
                            .chain(),
                        portal::remove_portals,
                    ),
                )
                    .chain(),
                player::rotation,
                input::cursor_grab,
                input::cursor_ungrab,
                input::exit_on_primary_close,
                debug_info::portal_surface_gizmo,
                debug_info::portal_gizmo,
            ),
        )
        .add_systems(PreUpdate, portal::resize_portal_view_image)
        .add_systems(
            PostUpdate,
            (portal::move_portal_camera, portal::portal_camera_gizmo).chain(),
        )
        .run();
}

// //! Shows how to render to a texture. Useful for mirrors, UI, or exporting images.

// use std::f32::consts::PI;

// use bevy::{
//     prelude::*,
//     render::{
//         render_resource::{
//             Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
//         },
//         view::RenderLayers,
//     },
// };

// fn main() {
//     App::new()
//         .add_plugins(DefaultPlugins)
//         .add_systems(Startup, setup)
//         .add_systems(Update, (cube_rotator_system, rotator_system))
//         .run();
// }

// // Marks the first pass cube (rendered to a texture.)
// #[derive(Component)]
// struct FirstPassCube;

// // Marks the main pass cube, to which the texture is applied.
// #[derive(Component)]
// struct MainPassCube;

// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut images: ResMut<Assets<Image>>,
// ) {
//     let size = Extent3d {
//         width: 512,
//         height: 512,
//         ..default()
//     };

//     // This is the texture that will be rendered to.
//     let mut image = Image {
//         texture_descriptor: TextureDescriptor {
//             label: None,
//             size,
//             dimension: TextureDimension::D2,
//             format: TextureFormat::Bgra8UnormSrgb,
//             mip_level_count: 1,
//             sample_count: 1,
//             usage: TextureUsages::TEXTURE_BINDING
//                 | TextureUsages::COPY_DST
//                 | TextureUsages::RENDER_ATTACHMENT,
//             view_formats: &[],
//         },
//         ..default()
//     };

//     // fill image.data with zeroes
//     image.resize(size);

//     let image_handle = images.add(image);

//     let cube_handle = meshes.add(Cuboid::new(4.0, 4.0, 4.0));
//     let cube_material_handle = materials.add(StandardMaterial {
//         base_color: Color::rgb(0.8, 0.7, 0.6),
//         reflectance: 0.02,
//         unlit: false,
//         ..default()
//     });

//     // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
//     let first_pass_layer = RenderLayers::layer(1);

//     // The cube that will be rendered to the texture.
//     commands.spawn((
//         PbrBundle {
//             mesh: cube_handle,
//             material: cube_material_handle,
//             transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
//             ..default()
//         },
//         FirstPassCube,
//         first_pass_layer,
//     ));

//     // Light
//     // NOTE: we add the light to all layers so it affects both the rendered-to-texture cube, and the cube on which we display the texture
//     // Setting the layer to RenderLayers::layer(0) would cause the main view to be lit, but the rendered-to-texture cube to be unlit.
//     // Setting the layer to RenderLayers::layer(1) would cause the rendered-to-texture cube to be lit, but the main view to be unlit.
//     commands.spawn((
//         PointLightBundle {
//             transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
//             ..default()
//         },
//         RenderLayers::all(),
//     ));

//     commands.spawn((
//         Camera3dBundle {
//             camera: Camera {
//                 // render before the "main pass" camera
//                 order: -1,
//                 target: image_handle.clone().into(),
//                 clear_color: Color::WHITE.into(),
//                 ..default()
//             },
//             transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
//                 .looking_at(Vec3::ZERO, Vec3::Y),
//             ..default()
//         },
//         first_pass_layer,
//     ));

//     let cube_size = 4.0;
//     let cube_handle = meshes.add(Cuboid::new(cube_size, cube_size, cube_size));

//     // This material has the texture that has been rendered.
//     let material_handle = materials.add(StandardMaterial {
//         base_color_texture: Some(image_handle),
//         reflectance: 0.02,
//         unlit: false,
//         ..default()
//     });

//     // Main pass cube, with material containing the rendered first pass texture.
//     commands.spawn((
//         PbrBundle {
//             mesh: cube_handle,
//             material: material_handle,
//             transform: Transform::from_xyz(0.0, 0.0, 1.5)
//                 .with_rotation(Quat::from_rotation_x(-PI / 5.0)),
//             ..default()
//         },
//         MainPassCube,
//     ));

//     // The main pass camera.
//     commands.spawn(Camera3dBundle {
//         transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
//         ..default()
//     });
// }

// /// Rotates the inner cube (first pass)
// fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<FirstPassCube>>) {
//     for mut transform in &mut query {
//         transform.rotate_x(1.5 * time.delta_seconds());
//         transform.rotate_z(1.3 * time.delta_seconds());
//     }
// }

// /// Rotates the outer cube (main pass)
// fn cube_rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<MainPassCube>>) {
//     for mut transform in &mut query {
//         transform.rotate_x(1.0 * time.delta_seconds());
//         transform.rotate_y(0.7 * time.delta_seconds());
//     }
// }
