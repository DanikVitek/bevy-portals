use bevy::{
    ecs::query::QuerySingleError,
    prelude::*,
    render::render_resource::{
        AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat,
        TextureUsages,
    },
    window::{PrimaryWindow, WindowResized},
};
use bevy_rapier3d::{
    geometry::{CollisionGroups, Group},
    pipeline::QueryFilter,
    plugin::RapierContext,
};

use crate::{domain::player::PLAYER_COLLISION_GROUP, resource::Controls};

use super::player::PlayerCamera;

pub const DEFAULT_PORTAL_SIZE: Vec2 = Vec2::new(1., 2.);
pub const PORTAL_RAY_COLLISION_GROUP: Group = Group::GROUP_5;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct PortalSurface {
    pub size: Vec2,
}

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct PortalCamera;

#[derive(Debug, Clone, Copy, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Portal {
    pair: Option<Entity>,
}

pub trait PortalKind: Component + Copy {
    type Pair: PortalKind<Pair = Self>;

    fn color() -> Color;
}

#[derive(Debug, Clone, Copy, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Portal1;

#[derive(Debug, Clone, Copy, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Portal2;

impl PortalKind for Portal1 {
    type Pair = Portal2;

    fn color() -> Color {
        Color::rgb(0.6, 0.7, 0.8)
    }
}

impl PortalKind for Portal2 {
    type Pair = Portal1;

    fn color() -> Color {
        Color::rgb(0.8, 0.7, 0.6)
    }
}

pub(super) trait AppExt {
    fn register_portal_types(&mut self) -> &mut Self;
}

impl AppExt for App {
    fn register_portal_types(&mut self) -> &mut Self {
        self.register_type::<PortalSurface>()
            .register_type::<Portal>()
            .register_type::<Portal1>()
            .register_type::<Portal2>()
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub struct SpawnPortal<P: PortalKind> {
    portal_kind: P,
    transform: Transform,
}

#[allow(clippy::type_complexity)]
pub fn shoot_portal(
    controls: Res<Controls>,
    camera_q: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
    portal_surface_q: Query<(&GlobalTransform, &PortalSurface)>,
    rapier_ctx: Res<RapierContext>,
    mut spawn_portal1_evt: EventWriter<SpawnPortal<Portal1>>,
    mut spawn_portal2_evt: EventWriter<SpawnPortal<Portal2>>,
) {
    if !controls.shoot1 && !controls.shoot2 {
        return;
    }

    println!("Shooting portal\n");

    let (camera, camera_transform) = camera_q.single();
    let viewport_center = camera.logical_viewport_size().unwrap() / 2.;

    let (ray_origin, ray_dir, max_toi) = {
        // inlined body of `Camera::viewport_to_world`
        let mut viewport_position = viewport_center;
        let Some(target_size) = camera.logical_viewport_size() else {
            return;
        };
        // Flip the Y co-ordinate origin from the top to the bottom.
        viewport_position.y = target_size.y - viewport_position.y;
        let ndc = viewport_position * 2. / target_size - Vec2::ONE;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
        let world_near_plane = ndc_to_world.project_point3(ndc.extend(1.));
        // Using EPSILON because an ndc with Z = 0 returns NaNs.
        let world_far_plane = ndc_to_world.project_point3(ndc.extend(f32::EPSILON));

        let ray_line = world_far_plane - world_near_plane;
        (world_near_plane, ray_line.normalize(), ray_line.length())
    };

    let Some((entity, distance)) = rapier_ctx.cast_ray(
        ray_origin,
        ray_dir,
        max_toi,
        true,
        QueryFilter::new().groups(CollisionGroups::new(
            PORTAL_RAY_COLLISION_GROUP,
            PLAYER_COLLISION_GROUP.complement() & PORTAL_RAY_COLLISION_GROUP.complement(),
        )),
    ) else {
        return;
    };

    let Ok((transform, &PortalSurface { size })) = portal_surface_q.get(entity) else {
        return;
    };

    let point = ray_origin + ray_dir * distance;
    println!("Intersect with plane {transform:?} at point {point}\n");

    let point_on_plane = transform.affine().inverse().transform_point3(point).xy();
    println!("Point on the plane: {point_on_plane}\n");

    let half_size = size * transform.to_scale_rotation_translation().0.xy() / 2.;

    if point_on_plane.abs().cmpgt(half_size).any() {
        // point is outside the portal surface
        return;
    }

    let mut portal_transform = transform.compute_transform();
    let clamped_point = transform.affine().transform_point3(
        point_on_plane
            .clamp(
                -half_size + DEFAULT_PORTAL_SIZE / 2.,
                half_size - DEFAULT_PORTAL_SIZE / 2.,
            )
            .extend(0.),
    );
    portal_transform.translation +=
        (clamped_point - transform.translation()) + transform.back() * 0.01;

    if controls.shoot1 {
        spawn_portal1_evt.send(SpawnPortal {
            portal_kind: Portal1,
            transform: portal_transform,
        });
    } else if controls.shoot2 {
        spawn_portal2_evt.send(SpawnPortal {
            portal_kind: Portal2,
            transform: portal_transform,
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_portal<P: PortalKind>(
    mut spawn_portal_events: EventReader<SpawnPortal<P>>,
    portal_q: Query<Entity, With<P>>,
    mut pair_portal_q: Query<(Entity, &mut Portal), With<P::Pair>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut portal_view_materials: ResMut<Assets<PortalViewMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    portal_cam_q: Query<(&Parent, &Camera), With<PortalCamera>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_q.single();

    for SpawnPortal {
        portal_kind,
        transform,
    } in spawn_portal_events.read().copied()
    {
        println!("Spawning {}", std::any::type_name::<P>());

        match portal_q.get_single() {
            Ok(portal) => commands.entity(portal).despawn_recursive(),
            Err(err @ QuerySingleError::MultipleEntities(_)) => unreachable!("{err}"),
            Err(QuerySingleError::NoEntities(_)) => {}
        }

        let pair = pair_portal_q.iter_mut().next();

        let portal_view_image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: window.physical_width(),
                    height: window.physical_height(),
                    ..Default::default()
                },
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..Default::default()
        };

        let portal_view_image_handle = images.add(portal_view_image);

        let mut new_portal = commands.spawn((
            Portal {
                pair: pair.iter().map(|(entity, _)| entity).copied().next(),
            },
            portal_kind,
            TransformBundle::from_transform(transform),
            VisibilityBundle::default(),
            meshes.add(Plane3d::new(Vec3::Z).mesh().size(1., 2.)),
        ));
        new_portal.with_children(|child| {
            child.spawn((
                PortalCamera,
                Camera3dBundle {
                    camera: Camera {
                        order: -1,
                        target: portal_view_image_handle.clone().into(),
                        clear_color: Color::GRAY.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ));
        });

        match pair {
            Some((pair_entity, mut pair_portal)) => {
                _ = pair_portal.pair.replace(new_portal.id());

                new_portal.insert(
                    portal_view_materials.add(PortalViewMaterial {
                        portal_view: portal_cam_q
                            .iter()
                            .find_map(|(parent, cam)| {
                                (parent.get() == pair_entity)
                                    .then(|| cam.target.as_image().unwrap())
                            })
                            .unwrap()
                            .to_owned(),
                    }),
                );

                commands
                    .entity(pair_entity)
                    .remove::<Handle<StandardMaterial>>()
                    .insert(portal_view_materials.add(PortalViewMaterial {
                        portal_view: portal_view_image_handle,
                    }));
            }
            None => {
                new_portal.insert(standard_materials.add(StandardMaterial {
                    base_color: P::color(),
                    ..Default::default()
                }));
            }
        }
    }
}

#[derive(Debug, Clone, AsBindGroup, Asset, TypePath)]
pub struct PortalViewMaterial {
    #[texture(0)]
    #[sampler(1)]
    portal_view: Handle<Image>,
}

impl Material for PortalViewMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/portal_view.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(1.)
    }
}

pub fn move_portal_camera(
    portal_q: Query<(Entity, &Children, &GlobalTransform, &Portal)>,
    mut portal_camera_q: Query<(&GlobalTransform, &mut Transform), With<PortalCamera>>,
    player_camera_q: Query<&GlobalTransform, With<PlayerCamera>>,
) {
    let player_camera_gt = player_camera_q.single();
    for (portal, children, portal_gt, pair) in
        portal_q
            .iter()
            .filter_map(|(portal, children, gt, &Portal { pair })| {
                pair.map(|pair| (portal, children, gt, pair))
            })
    {
        let (_, mut portal_cam_t) = portal_camera_q
            .get_mut(
                children
                    .iter()
                    .copied()
                    .find(|child| *child == portal)
                    .unwrap(),
            )
            .unwrap();
        let (_, _, pair_portal_gt, _) = portal_q.get(pair).unwrap();

        let (new_scale, new_rotation, new_translation) = (portal_gt.compute_matrix()
            * pair_portal_gt.compute_matrix().inverse()
            * player_camera_gt.compute_matrix().inverse())
        .to_scale_rotation_translation();
        portal_cam_t.scale = new_scale;
        portal_cam_t.translation = new_translation;
        portal_cam_t.rotation = new_rotation;
    }
}

pub fn resize_portal_view_image(
    mut resize_events: EventReader<WindowResized>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    portal_cam_q: Query<&Camera, With<PortalCamera>>,
    mut images: ResMut<Assets<Image>>,
) {
    for &WindowResized { window, .. } in resize_events.read() {
        let Ok(window) = window_q.get(window) else {
            return;
        };
        for camera in portal_cam_q.iter() {
            let Some(image) = images.get_mut(camera.target.as_image().unwrap().id()) else {
                continue;
            };
            image.resize(Extent3d {
                width: window.physical_width(),
                height: window.physical_height(),
                ..Default::default()
            })
        }
    }
}
