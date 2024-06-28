#![allow(clippy::type_complexity)]

use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::{
    component::{Player, Velocity},
    resource::{Controls, MouseSensitivity},
};

pub fn movement(
    time: Res<Time>,
    controls: Res<Controls>,
    rapier_config: Res<RapierConfiguration>,
    mut player_query: Query<
        (
            &mut Velocity,
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
            &Transform,
        ),
        With<Player>,
    >,
) {
    /// m/s
    const WALK_SPEED: f32 = 1.42;
    const RUN_SPEED: f32 = WALK_SPEED * 2.;
    /// 1/second
    const DECAY: f32 = 5.;
    const TERMINAL_VELOCITY: f32 = 50.;
    /// m/s
    const JUMP_SPEED: f32 = 3.;

    let (mut velocity, mut controller, controller_output, transform) = player_query.single_mut();
    let rotation_angle = transform.rotation.to_euler(EulerRot::YXZ).0;
    let target_velocity = controls
        .to_direction()
        .rotate(Vec2::new(rotation_angle.cos(), -rotation_angle.sin()))
        * (if controls.run { RUN_SPEED } else { WALK_SPEED });

    velocity.exp_decay_horizontal(target_velocity, DECAY, time.delta_seconds());
    let grounded = controller_output.map(|o| o.grounded).unwrap_or_default();
    velocity.0.y = if grounded {
        if controls.jump {
            JUMP_SPEED
        } else {
            0.0
        }
    } else {
        (velocity.0.y + rapier_config.gravity.y * time.delta_seconds()).max(-TERMINAL_VELOCITY)
    };

    controller.translation =
        (velocity.0.length() > 0.001).then_some(velocity.0 * time.delta_seconds());
}

pub fn rotation(
    time: Res<Time>,
    sensitivity: Res<MouseSensitivity>,
    mut input: EventReader<MouseMotion>,
    mut player_query: Query<(Entity, &mut Transform), (With<Player>, Without<Camera3d>)>,
    mut camera_query: Query<(&Parent, &mut Transform), (With<Camera3d>, Without<Player>)>,
) {
    let (player, mut player_transform) = player_query.single_mut();
    let mut camera_transform = camera_query
        .iter_mut()
        .find_map(|(parent, transform)| (parent.get() == player).then_some(transform))
        .unwrap();
    let (mut camera_pitch, _, _) = camera_transform.rotation.to_euler(EulerRot::XYZ);

    for MouseMotion {
        delta: Vec2 { x, y },
    } in input.read().copied()
    {
        player_transform.rotate_y(-x * time.delta_seconds() * sensitivity.0);

        camera_pitch = (camera_pitch - y * time.delta_seconds() * sensitivity.0)
            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    }
    camera_transform.rotation = Quat::from_rotation_x(camera_pitch);
}
