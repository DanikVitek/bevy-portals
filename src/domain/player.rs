#![allow(clippy::type_complexity)]

use bevy::{input::mouse::MouseMotion, prelude::*, render::view::RenderLayers};
#[cfg(feature = "debug")]
use bevy_editor_pls::default_windows::cameras::EDITOR_RENDER_LAYER;
use bevy_rapier3d::prelude::*;

use crate::{
    resource::{Controls, MouseSensitivity},
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
const WALK_SPEED: f32 = 1.7;
/// m/s
const RUN_SPEED: f32 = WALK_SPEED * 2.;
/// 1/second
const DECAY: f32 = 10.;
/// m/s
const TERMINAL_VELOCITY: f32 = 50.;
/// m/s
const JUMP_SPEED: f32 = 5.;

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
) {
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
            RigidBody::KinematicPositionBased,
            Collider::capsule_y(PLAYER_HEIGHT / 2. - PLAYER_RADIUS, PLAYER_RADIUS),
            KinematicCharacterController {
                custom_mass: Some(60.), // kg
                apply_impulse_to_dynamic_bodies: true,
                ..Default::default()
            },
            CollisionGroups::new(PLAYER_COLLISION_GROUP, Group::all()),
            #[cfg(feature = "debug")]
            RenderLayers::from_layers(&[PLAYER_RENDER_LAYER, EDITOR_RENDER_LAYER]),
            #[cfg(not(feature = "debug"))]
            RenderLayers::layer(PLAYER_RENDER_LAYER),
        ))
        .with_children(|child| {
            child.spawn((
                Name::new("FPS Camera"),
                PlayerCamera,
                Camera3dBundle {
                    transform: Transform::from_xyz(0., EYES_HEIGHT / 2., 0.),
                    projection: PerspectiveProjection {
                        fov: std::f32::consts::FRAC_PI_2, // 90 degrees
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
            ));

            const GROUND_SENSOR_HEIGHT: f32 = 0.1;
            child.spawn((
                Name::new("Ground Sensor"),
                GroundSensor,
                Collider::cylinder(GROUND_SENSOR_HEIGHT / 2., PLAYER_RADIUS),
                Sensor,
                TransformBundle::from_transform(Transform::from_xyz(
                    0.,
                    -(PLAYER_HEIGHT / 2. + GROUND_SENSOR_HEIGHT / 3.),
                    0.,
                )),
                CollisionGroups::new(PLAYER_COLLISION_GROUP, PLAYER_COLLISION_GROUP.complement()),
                SolverGroups::new(Group::empty(), Group::empty()),
                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
            ));
        });
}

pub fn movement(
    time: Res<Time>,
    controls: Res<Controls>,
    rapier_config: Res<RapierConfiguration>,
    rapier_context: Res<RapierContext>,
    mut player_q: Query<
        (
            &mut Velocity,
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
            &Transform,
            &mut Grounded,
        ),
        With<Player>,
    >,
    ground_sensor_q: Query<Entity, With<GroundSensor>>,
) {
    let (mut velocity, mut controller, controller_output, player_transform, mut grounded) =
        player_q.single_mut();
    let ground_sensor = ground_sensor_q.single();

    let rotation_angle = player_transform.rotation.to_euler(EulerRot::YXZ).0;
    let target_velocity = controls
        .to_direction()
        .rotate(Vec2::new(rotation_angle.cos(), -rotation_angle.sin()))
        * (if controls.run { RUN_SPEED } else { WALK_SPEED });

    velocity.exp_decay_horizontal(target_velocity, DECAY, time.delta_seconds());
    grounded.0 = controller_output.map(|o| o.grounded).unwrap_or_default()
        || rapier_context
            .intersection_pairs_with(ground_sensor)
            .any(|(_, _, intersect)| intersect);
    velocity.0.y = if grounded.0 {
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
