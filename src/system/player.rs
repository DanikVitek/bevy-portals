use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::control::KinematicCharacterController;

use crate::{
    component::{HorizontalVelocity, Player},
    resource::Controls,
};

pub fn movement(
    time: Res<Time>,
    controls: Res<Controls>,
    mut player_query: Query<
        (
            &mut HorizontalVelocity,
            &mut KinematicCharacterController,
            &Transform,
        ),
        With<Player>,
    >,
) {
    /// meters per second
    const PLAYER_SPEED: f32 = 1.42;

    let (mut velocity, mut controller, transform) = player_query.single_mut();
    let rotation_angle = transform.rotation.to_euler(EulerRot::YXZ).0;
    let target_velocity = controls
        .to_direction()
        .rotate(Vec2::new(rotation_angle.cos(), -rotation_angle.sin()))
        * PLAYER_SPEED;

    velocity.0 = velocity
        .exp_decay(target_velocity, 5.0, time.delta_seconds())
        .0;

    controller.translation = Some((velocity.0 * time.delta_seconds()).extend(0.0).xzy());
}

pub fn rotation(
    time: Res<Time>,
    mut input: EventReader<MouseMotion>,
    mut player_query: Query<(Entity, &mut Transform), (With<Player>, Without<Camera3d>)>,
    mut camera_query: Query<(&Parent, &mut Transform), (With<Camera3d>, Without<Player>)>,
) {
    for MouseMotion {
        delta: Vec2 { x, y },
    } in input.read().copied()
    {
        let (player, mut player_transform) = player_query.single_mut();
        player_transform.rotate_y(-x * time.delta_seconds());

        let mut camera_transform = camera_query
            .iter_mut()
            .find_map(|(parent, transform)| (parent.get() == player).then_some(transform))
            .unwrap();
        let (mut camera_pitch, _, _) = camera_transform.rotation.to_euler(EulerRot::XYZ);
        camera_pitch = (camera_pitch - y * time.delta_seconds())
            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
        camera_transform.rotation = Quat::from_rotation_x(camera_pitch);
    }
}
