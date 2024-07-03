use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::portal::{PortalSurface, PORTAL_RAY_COLLISION_GROUP};

pub const STATIC_COLLISION_GROUP: Group = Group::GROUP_1;
pub const DYNAMIC_COLLISION_GROUP: Group = Group::GROUP_3;
pub const PORTAL_SURFACE_COLLISION_GROUP: Group = Group::GROUP_4;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Ground"),
        Collider::halfspace(Vect::Y).unwrap(),
        RigidBody::Fixed,
        CollisionGroups::new(STATIC_COLLISION_GROUP, Group::all()),
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(128., 128.)),
            material: materials.add(Color::GRAY),
            ..Default::default()
        },
    ));

    commands.spawn((
        Name::new("Cube"),
        Collider::cuboid(0.5, 0.5, 0.5), // m^3
        RigidBody::Dynamic,
        CollisionGroups::new(DYNAMIC_COLLISION_GROUP, Group::all()),
        ColliderMassProperties::Mass(30.), // kg
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1., 1., 1.)),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(1.5, 0.5, 1.5),
            ..Default::default()
        },
    ));

    commands
        .spawn((
            Name::new("Wall 1"),
            Collider::cuboid(0.1, 1., 2.),
            RigidBody::Fixed,
            CollisionGroups::new(STATIC_COLLISION_GROUP, Group::all()),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.2, 2., 4.)),
                material: materials.add(Color::GRAY),
                transform: Transform::from_xyz(5., 1., 2.),
                ..Default::default()
            },
        ))
        .with_children(|child| {
            child.spawn((
                PortalSurface {
                    size: Vec2::new(4., 2.),
                },
                TransformBundle::from_transform(Transform {
                    translation: Vec3::new(0.1, 0., 0.),
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    ..Default::default()
                }),
                Collider::cuboid(2., 1., 0.01),
                Sensor,
                CollisionGroups::new(PORTAL_SURFACE_COLLISION_GROUP, PORTAL_RAY_COLLISION_GROUP)
            ));

            child.spawn((
                PortalSurface {
                    size: Vec2::new(4., 2.),
                },
                TransformBundle::from_transform(Transform {
                    translation: Vec3::new(-0.1, 0., 0.),
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 * 3.),
                    ..Default::default()
                }),
                Collider::cuboid(2., 1., 0.01),
                Sensor,
                CollisionGroups::new(PORTAL_SURFACE_COLLISION_GROUP, PORTAL_RAY_COLLISION_GROUP)
            ));
        });

    commands
        .spawn((
            Name::new("Wall 2"),
            Collider::cuboid(0.1, 1., 2.),
            RigidBody::Fixed,
            CollisionGroups::new(STATIC_COLLISION_GROUP, Group::all()),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.2, 2., 4.)),
                material: materials.add(Color::GRAY),
                transform: Transform::from_xyz(5., 1., 10.)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)),
                ..Default::default()
            },
        ))
        .with_children(|child| {
            child.spawn((
                PortalSurface {
                    size: Vec2::new(4., 2.),
                },
                TransformBundle::from_transform(Transform {
                    translation: Vec3::new(0.1, 0., 0.),
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    ..Default::default()
                }),
                Collider::cuboid(2., 1., 0.01),
                Sensor,
                CollisionGroups::new(PORTAL_SURFACE_COLLISION_GROUP, PORTAL_RAY_COLLISION_GROUP)
            ));

            child.spawn((
                PortalSurface {
                    size: Vec2::new(4., 2.),
                },
                TransformBundle::from_transform(Transform {
                    translation: Vec3::new(-0.1, 0., 0.),
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 * 3.),
                    ..Default::default()
                }),
                Collider::cuboid(2., 1., 0.01),
                Sensor,
                CollisionGroups::new(PORTAL_SURFACE_COLLISION_GROUP, PORTAL_RAY_COLLISION_GROUP)
            ));
        });

    commands
        .spawn((
            Name::new("Wall 3"),
            Collider::cuboid(0.1, 1., 2.),
            RigidBody::Fixed,
            CollisionGroups::new(STATIC_COLLISION_GROUP, Group::all()),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.2, 2., 4.)),
                material: materials.add(Color::GRAY),
                transform: Transform::from_xyz(20., 3., 15.).with_rotation(
                    Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)
                        * Quat::from_rotation_x(45_f32.to_radians())
                        * Quat::from_rotation_z(45_f32.to_radians()),
                ),
                ..Default::default()
            },
        ))
        .with_children(|child| {
            child.spawn((
                PortalSurface {
                    size: Vec2::new(4., 2.),
                },
                TransformBundle::from_transform(Transform {
                    translation: Vec3::new(0.1, 0., 0.),
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    ..Default::default()
                }),
                Collider::cuboid(2., 1., 0.01),
                Sensor,
                CollisionGroups::new(PORTAL_SURFACE_COLLISION_GROUP, PORTAL_RAY_COLLISION_GROUP)
            ));

            child.spawn((
                PortalSurface {
                    size: Vec2::new(4., 2.),
                },
                TransformBundle::from_transform(Transform {
                    translation: Vec3::new(-0.1, 0., 0.),
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 * 3.),
                    ..Default::default()
                }),
                Collider::cuboid(2., 1., 0.01),
                Sensor,
                CollisionGroups::new(PORTAL_SURFACE_COLLISION_GROUP, PORTAL_RAY_COLLISION_GROUP)
            ));
        });

    // Lights

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::default().looking_at(-2. * Vec3::Y + Vec3::X + Vec3::Z, Vec3::Y),
        ..Default::default()
    });
}
