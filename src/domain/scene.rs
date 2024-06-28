use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn setup(mut commands: Commands) {
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
