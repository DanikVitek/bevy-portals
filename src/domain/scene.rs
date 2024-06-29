use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Ground"),
        Collider::halfspace(Vect::Y).unwrap(),
        RigidBody::Fixed,
        CollisionGroups::new(Group::GROUP_1, Group::all()),
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(128., 128.)),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5)),
            ..Default::default()
        },
    ));

    commands.spawn((
        Name::new("Cube"),
        Collider::cuboid(0.5, 0.5, 0.5), // m^3
        RigidBody::Dynamic,
        CollisionGroups::new(Group::GROUP_3, Group::all()),
        ColliderMassProperties::Mass(30.), // kg
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1., 1., 1.)),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(1.5, 0.5, 1.5),
            ..Default::default()
        },
    ));

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(5.0, 5.0, 5.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::default().looking_at(-Vec3::Y, Vec3::Z),
        ..Default::default()
    });
}
