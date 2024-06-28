use bevy::{
    prelude::*,
    render::view::RenderLayers,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;

use crate::{
    component::{DebugInfoRoot, DebugInfoText, Player, Velocity},
    PLAYER_RENDER_LAYER,
};

pub fn player(mut commands: Commands) {
    const PLAYER_HEIGHT: f32 = 1.75;
    const PLAYER_RADIUS: f32 = 0.3;
    const VIEW_HEIGHT: f32 = 1.5;

    commands
        .spawn((
            Player,
            Velocity(Vec3::ZERO),
            SpatialBundle::from_transform(Transform::from_xyz(0., PLAYER_HEIGHT / 2.0, 0.)),
            RigidBody::KinematicPositionBased,
            Collider::capsule(
                Vec3::new(0., PLAYER_RADIUS, 0.),
                Vec3::new(0., PLAYER_HEIGHT - PLAYER_RADIUS, 0.),
                PLAYER_RADIUS,
            ),
            KinematicCharacterController {
                custom_mass: Some(60.), // kg
                apply_impulse_to_dynamic_bodies: true,
                ..Default::default()
            },
            RenderLayers::layer(PLAYER_RENDER_LAYER),
        ))
        .with_children(|child| {
            child.spawn((
                Camera3dBundle {
                    transform: Transform::from_xyz(0., VIEW_HEIGHT / 2., 0.),
                    projection: PerspectiveProjection {
                        fov: 90_f32.to_radians(),
                        ..Default::default()
                    }
                    .into(),
                    // dither: DebandDither::Enabled,
                    ..Default::default()
                },
                RenderLayers::all().without(PLAYER_RENDER_LAYER),
            ));
        });
}

pub fn cursor_grab(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = window_query.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}

pub fn scene(mut commands: Commands) {
    commands.spawn((
        Collider::cuboid(2.5, 0.1, 2.5),
        TransformBundle::from(Transform::from_xyz(0., -0.2, 0.)),
    ));

    commands.spawn((
        Collider::cuboid(0.25, 0.25, 0.25), // m^3
        RigidBody::Dynamic,
        ColliderMassProperties::Mass(30.), // kg
        TransformBundle::from(Transform::from_xyz(2., 0.5, 0.)),
    ));
}

pub fn debug_info(mut commands: Commands) {
    commands
        .spawn((
            DebugInfoRoot,
            NodeBundle {
                z_index: ZIndex::Global(i32::MAX),
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(1.),
                    top: Val::Percent(1.),
                    bottom: Val::Auto,
                    left: Val::Auto,
                    padding: UiRect::all(Val::Px(4.)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .with_children(|child| {
            child.spawn((
                DebugInfoText,
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new("Player is grounded: ", TextStyle::default()),
                        TextSection::new("N/A", TextStyle::default()),
                        TextSection::new("\n", TextStyle::default()),
                    ]),
                    ..Default::default()
                },
            ));
        });
}
