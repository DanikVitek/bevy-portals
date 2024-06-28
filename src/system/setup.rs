use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;

use crate::component::{HorizontalVelocity, Player};

pub fn setup_player(mut commands: Commands) {
    const PLAYER_HEIGHT: f32 = 1.75;
    const PLAYER_RADIUS: f32 = 0.3;
    const VIEW_HEIGHT: f32 = 1.5;

    commands
        .spawn((
            Player,
            HorizontalVelocity(Vec2::ZERO),
            SpatialBundle::from_transform(Transform::from_xyz(0.0, PLAYER_HEIGHT / 2.0, 0.0)),
            RigidBody::KinematicPositionBased,
            Collider::capsule(
                Vec3::new(0.0, PLAYER_RADIUS, 0.0),
                Vec3::new(0.0, PLAYER_HEIGHT - PLAYER_RADIUS, 0.0),
                PLAYER_RADIUS,
            ),
            KinematicCharacterController {
                custom_mass: Some(60.0),
                offset: CharacterLength::Absolute(0.01),
                ..Default::default()
            },
        ))
        .with_children(|child| {
            child.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, VIEW_HEIGHT / 2.0, 0.0),
                // dither: DebandDither::Enabled,
                ..Default::default()
            });
        });
}

pub fn cursor_grab(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = window_query.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}

fn cursor_ungrab(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_windows.single_mut();

    primary_window.cursor.grab_mode = CursorGrabMode::None;
    primary_window.cursor.visible = true;
}

pub fn setup_physics(mut commands: Commands) {
    /* Create the ground. */
    commands
        .spawn(Collider::cuboid(2.5, 0.1, 2.5))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -0.2, 0.0)));
}
