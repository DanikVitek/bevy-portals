#![allow(clippy::type_complexity)]

use bevy::{input::mouse::MouseMotion, prelude::*, render::view::RenderLayers};
#[cfg(feature = "debug")]
use bevy_editor_pls::default_windows::cameras::EDITOR_RENDER_LAYER;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    builtins::{TnuaBuiltinJump, TnuaBuiltinWalk},
    controller::{TnuaController, TnuaControllerBundle},
};
use bevy_tnua_rapier3d::{TnuaRapier3dIOBundle, TnuaRapier3dSensorShape};

use crate::{
    resource::{Controls, Fov, MouseSensitivity},
    ExpDecay,
};

pub const PLAYER_RENDER_LAYER: u8 = 1;
pub const PLAYER_COLLISION_GROUP: Group = Group::GROUP_2;

/// m
const PLAYER_HEIGHT: f32 = 1.75;
/// m
const PLAYER_RADIUS: f32 = 0.3;
/// m
const EYES_HEIGHT: f32 = PLAYER_HEIGHT - 2. * PLAYER_RADIUS;

/// m/s
const WALK_SPEED: f32 = 5.;
/// m/s
const RUN_SPEED: f32 = WALK_SPEED * 2.;
/// m
const JUMP_HEIGHT: f32 = 3.;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct Grounded(pub bool);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GroundSensor;

#[derive(Clone, Copy, Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct Velocity(pub Vec3);

impl Velocity {
    pub fn exp_decay_horizontal(&mut self, target: Vec2, decay: f32, delta: f32) {
        let Self(Vec3 { x, y: _, z }) = self;
        *x = x.exp_decay(target.x, decay, delta);
        *z = z.exp_decay(target.y, decay, delta);
    }
}

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerCamera;

pub(super) trait AppExt {
    fn register_player_types(&mut self) -> &mut Self;
}

impl AppExt for App {
    fn register_player_types(&mut self) -> &mut Self {
        self.register_type::<Player>()
            .register_type::<Grounded>()
            .register_type::<GroundSensor>()
            .register_type::<Velocity>()
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    fov: Res<Fov>,
) -> Entity {
    let camera = commands
        .spawn((
            Name::new("FPS Camera"),
            PlayerCamera,
            Camera3dBundle {
                transform: Transform::from_xyz(0., EYES_HEIGHT / 2., 0.),
                projection: PerspectiveProjection {
                    fov: fov.radians(),
                    ..Default::default()
                }
                .into(),
                // dither: DebandDither::Enabled,
                ..Default::default()
            },
            #[cfg(feature = "debug")]
            const {
                RenderLayers::all()
                    .without(PLAYER_RENDER_LAYER)
                    .without(EDITOR_RENDER_LAYER)
            },
            #[cfg(not(feature = "debug"))]
            const { RenderLayers::all().without(PLAYER_RENDER_LAYER) },
        ))
        .id();
    commands
        .spawn((
            Name::new("Player"),
            Player,
            Grounded::default(),
            Velocity::default(),
            PbrBundle {
                mesh: meshes.add(Mesh::from(Capsule3d::new(
                    PLAYER_RADIUS,
                    PLAYER_HEIGHT - 2. * PLAYER_RADIUS,
                ))),
                material: materials.add(Color::CYAN),
                transform: Transform::from_xyz(0., PLAYER_HEIGHT, 0.),
                ..Default::default()
            },
            RigidBody::Dynamic,
            Collider::capsule_y(PLAYER_HEIGHT / 2. - PLAYER_RADIUS, PLAYER_RADIUS),
            ColliderMassProperties::Mass(60.0),
            TnuaRapier3dIOBundle::default(),
            TnuaControllerBundle::default(),
            LockedAxes::ROTATION_LOCKED,
            TnuaRapier3dSensorShape(Collider::cylinder(0., PLAYER_RADIUS - 0.01)),
            CollisionGroups::new(PLAYER_COLLISION_GROUP, Group::all()),
            #[cfg(feature = "debug")]
            RenderLayers::from_layers(&[PLAYER_RENDER_LAYER, EDITOR_RENDER_LAYER]),
            #[cfg(not(feature = "debug"))]
            const { RenderLayers::layer(PLAYER_RENDER_LAYER) },
        ))
        .add_child(camera);

    camera
}

pub fn movement(
    controls: Res<Controls>,
    mut player_q: Query<(&GlobalTransform, &mut TnuaController), With<Player>>,
) {
    let Ok((player_gt, mut controller)) = player_q.get_single_mut() else {
        return;
    };

    let rotation_angle = Quat::from_affine3(&player_gt.affine())
        .to_euler(EulerRot::YXZ)
        .0;
    let speed = if controls.run { RUN_SPEED } else { WALK_SPEED };
    let desired_velocity = controls
        .to_direction()
        .rotate(Vec2::new(rotation_angle.cos(), -rotation_angle.sin()))
        * speed;
    let desired_velocity = Vec3::new(desired_velocity.x, 0., desired_velocity.y);

    controller.basis(TnuaBuiltinWalk {
        desired_velocity,
        // acceleration: speed * 0.9,
        float_height: PLAYER_HEIGHT / 2. + 0.005,
        // air_acceleration: speed * 0.5,
        ..Default::default()
    });

    if controls.jump {
        controller.action(TnuaBuiltinJump {
            height: JUMP_HEIGHT,
            ..Default::default()
        });
    }
}

pub fn rotation(
    time: Res<Time>,
    sensitivity: Res<MouseSensitivity>,
    mut input: EventReader<MouseMotion>,
    mut player_q: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera_q: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let mut player_transform = player_q.single_mut();
    let mut camera_transform = camera_q.single_mut();
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
