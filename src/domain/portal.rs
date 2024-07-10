use bevy::{
    core_pipeline::{
        core_3d::graph::Core3d,
        tonemapping::{DebandDither, Tonemapping},
    },
    ecs::query::QuerySingleError,
    math::{AspectRatio, Vec3A},
    pbr::PbrProjectionPlugin,
    prelude::*,
    render::{
        camera::{CameraMainTextureUsages, CameraProjection, CameraRenderGraph, Exposure},
        primitives::Frustum,
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
        view::{ColorGrading, VisibleEntities},
    },
    window::{PrimaryWindow, WindowResized},
};
#[cfg(feature = "debug")]
use bevy_editor_pls::default_windows::cameras::EDITOR_RENDER_LAYER;
use bevy_rapier3d::{
    geometry::{CollisionGroups, Group},
    pipeline::QueryFilter,
    plugin::RapierContext,
};

use crate::{
    domain::{debug_info, input, player::PLAYER_COLLISION_GROUP},
    resource::{Controls, Fov},
};
#[cfg(feature = "debug")]
use crate::ALL_RENDER_LAYERS;

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

    fn new() -> Self;

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

    fn new() -> Self {
        Self
    }

    fn color() -> Color {
        Color::srgb(0.6, 0.7, 0.8)
    }
}

impl PortalKind for Portal2 {
    type Pair = Portal1;

    fn new() -> Self {
        Self
    }

    fn color() -> Color {
        Color::srgb(0.8, 0.7, 0.6)
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
pub fn shoot_portal<P1: PortalKind<Pair = P2>, P2: PortalKind<Pair = P1>>(
    controls: Res<Controls>,
    camera_q: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
    portal_surface_q: Query<(&GlobalTransform, &PortalSurface)>,
    rapier_ctx: Res<RapierContext>,
    mut commands: Commands,
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
        let ndc_to_world = camera_transform.compute_matrix() * camera.clip_from_view().inverse();
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
        commands.trigger(SpawnPortal {
            portal_kind: P1::new(),
            transform: portal_transform,
        });
    } else if controls.shoot2 {
        commands.trigger(SpawnPortal {
            portal_kind: P2::new(),
            transform: portal_transform,
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_portal<P: PortalKind>(
    spawn_portal: Trigger<SpawnPortal<P>>,
    portal_q: Query<Entity, With<P>>,
    mut pair_portal_q: Query<(Entity, &mut Portal), With<P::Pair>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut portal_view_materials: ResMut<Assets<PortalViewMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    fov: Res<Fov>,
    mut portal_cam_q: Query<(&Parent, &mut Camera), With<PortalCamera>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_q.get_single() else {
        return;
    };

    let &SpawnPortal {
        portal_kind,
        transform,
    } = spawn_portal.event();

    println!(
        "Spawning {}",
        std::any::type_name::<P>().rsplit_once("::").unwrap().1
    );

    match portal_q.get_single() {
        Ok(portal) => commands.entity(portal).despawn_recursive(),
        Err(err @ QuerySingleError::MultipleEntities(_)) => unreachable!("{err}"),
        Err(QuerySingleError::NoEntities(_)) => {}
    }

    let pair = pair_portal_q.iter_mut().next();

    let portal_view_image = {
        let size = Extent3d {
            width: window.physical_width(),
            height: window.physical_height(),
            ..Default::default()
        };
        let mut img = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
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
        img.resize(size);
        img
    };

    let portal_view_image_handle = images.add(portal_view_image);

    let mut new_portal = commands.spawn((
        Name::new(std::any::type_name::<P>().rsplit_once("::").unwrap().1),
        Portal {
            pair: pair.iter().map(|(entity, _)| entity).copied().next(),
        },
        portal_kind,
        TransformBundle::from_transform(transform),
        VisibilityBundle::default(),
        meshes.add(Plane3d::new(Vec3::Z, Vec2::new(0.5, 1.))),
    ));
    new_portal.with_children(|child| {
        child.spawn((
            Name::new("Portal Camera"),
            PortalCamera,
            PortalCamera3dBundle {
                projection: PortalPerspectiveProjection {
                    fov: fov.radians(),
                    ..Default::default()
                },
                camera: Camera {
                    order: -1,
                    is_active: pair.is_some(),
                    target: portal_view_image_handle.clone().into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            #[cfg(feature = "debug")]
            ALL_RENDER_LAYERS.clone().without(EDITOR_RENDER_LAYER),
        ));
    });

    match pair {
        Some((pair_entity, mut pair_portal)) => {
            _ = pair_portal.pair.replace(new_portal.id());

            new_portal.insert(portal_view_materials.add(PortalViewMaterial {
                portal_view: portal_view_image_handle,
            }));

            let mut pair_cam = portal_cam_q
                .iter_mut()
                .find_map(|(parent, cam)| (parent.get() == pair_entity).then_some(cam))
                .unwrap();

            pair_cam.is_active = true;

            commands
                .entity(pair_entity)
                .remove::<Handle<StandardMaterial>>()
                .insert(portal_view_materials.add(PortalViewMaterial {
                    portal_view: pair_cam.target.as_image().unwrap().to_owned(),
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

/// The camera coordinate space is right-handed x-right, y-up, z-back.
/// This means "forward" is -Z.
#[derive(Bundle, Clone)]
pub struct PortalCamera3dBundle {
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub projection: PortalPerspectiveProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera_3d: Camera3d,
    pub tonemapping: Tonemapping,
    pub deband_dither: DebandDither,
    pub color_grading: ColorGrading,
    pub exposure: Exposure,
    pub main_texture_usages: CameraMainTextureUsages,
}

impl Default for PortalCamera3dBundle {
    fn default() -> Self {
        Self {
            camera_render_graph: CameraRenderGraph::new(Core3d),
            camera: Default::default(),
            projection: PortalPerspectiveProjection::default(),
            visible_entities: Default::default(),
            frustum: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            camera_3d: Default::default(),
            tonemapping: Default::default(),
            color_grading: Default::default(),
            exposure: Default::default(),
            main_texture_usages: Default::default(),
            deband_dither: DebandDither::Enabled,
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
        ShaderRef::from("shaders/portal_view.wgsl")
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(1.)
    }
}

pub fn move_portal_camera(
    portal_q: Query<(Entity, &GlobalTransform, &Portal)>,
    mut portal_cam_q: Query<
        (&mut Transform, &mut PortalPerspectiveProjection, &Parent),
        With<PortalCamera>,
    >,
    player_cam_q: Query<&GlobalTransform, With<PlayerCamera>>,
) {
    let player_cam_gt = player_cam_q.single();
    for (portal, portal_gt, pair) in portal_q
        .iter()
        .filter_map(|(portal, gt, &Portal { pair })| pair.map(|pair| (portal, gt, pair)))
    {
        let (mut portal_cam_t, mut projection) = portal_cam_q
            .iter_mut()
            .find_map(|(transform, projection, parent)| {
                (parent.get() == portal).then_some((transform, projection))
            })
            .unwrap();
        let (_, pair_portal_gt, _) = portal_q.get(pair).unwrap();

        let new_portal_cam_gt_mat = pair_portal_gt.compute_matrix()
            * Mat4::from_rotation_translation(
                Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI),
                Vec3::ZERO,
            )
            * portal_gt.compute_matrix().inverse()
            * player_cam_gt.compute_matrix();
        let (new_scale, new_rotation, new_translation) = (portal_gt.compute_matrix().inverse()
            * new_portal_cam_gt_mat)
            .to_scale_rotation_translation();

        portal_cam_t.scale = new_scale;
        portal_cam_t.rotation = new_rotation;
        portal_cam_t.translation = new_translation;

        // TODO: update near plane so that it is coplanar with the portal's mesh

        projection.near = (pair_portal_gt.translation() - new_portal_cam_gt_mat.w_axis.xyz())
            .length()
            .clamp(0.05, projection.far);
    }
}

pub fn portal_camera_gizmo(
    portal1_q: Query<&Portal1>,
    portal2_q: Query<&Portal2>,
    portal_cam_q: Query<
        (&GlobalTransform, &PortalPerspectiveProjection, &Parent),
        With<PortalCamera>,
    >,
    mut gizmos: Gizmos,
) {
    for (gt, projection, portal) in portal_cam_q.iter() {
        let color = portal1_q
            .get(portal.get())
            .map(|_| Portal1::color())
            .or_else(|_| portal2_q.get(portal.get()).map(|_| Portal2::color()))
            .unwrap();

        gizmos.cuboid(*gt, color);
        gizmos.arrow(gt.translation(), gt.translation() + *gt.forward(), color);

        gizmos.circle(
            gt.translation() + projection.near * gt.forward(),
            gt.forward(),
            0.5,
            color,
        );
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

pub fn remove_portals(
    controls: Res<Controls>,
    mut commands: Commands,
    portal_q: Query<Entity, With<Portal>>,
) {
    if !controls.remove_portals {
        return;
    }

    for portal in portal_q.iter() {
        commands.entity(portal).despawn_recursive();
    }
}

/// A 3D camera projection in which distant objects appear smaller than close objects.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct PortalPerspectiveProjection {
    /// The vertical field of view (FOV) in radians.
    ///
    /// Defaults to a value of π/4 radians or 45 degrees.
    pub fov: f32,

    /// The aspect ratio (width divided by height) of the viewing frustum.
    ///
    /// Bevy's [`camera_system`](crate::camera::camera_system) automatically
    /// updates this value when the aspect ratio of the associated window changes.
    ///
    /// Defaults to a value of `1.0`.
    pub aspect_ratio: f32,

    /// The distance from the camera in world units of the viewing frustum's near plane.
    ///
    /// Objects closer to the camera than this value will not be visible.
    ///
    /// Defaults to a value of `0.1`.
    pub near: f32,

    pub portal_frame: (f32, Plane3d),

    /// The distance from the camera in world units of the viewing frustum's far plane.
    ///
    /// Objects farther from the camera than this value will not be visible.
    ///
    /// Defaults to a value of `1000.0`.
    pub far: f32,
}

impl CameraProjection for PortalPerspectiveProjection {
    fn get_clip_from_view(&self) -> Mat4 {
        Mat4::perspective_infinite_reverse_rh(self.fov, self.aspect_ratio, self.near)
    }

    fn update(&mut self, width: f32, height: f32) {
        self.aspect_ratio = AspectRatio::new(width, height).into();
    }

    fn far(&self) -> f32 {
        self.far
    }

    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [Vec3A; 8] {
        // ratio between half the height of a viewport
        // rect and a distance to a rect with such height
        let tan_half_fov = (self.fov / 2.).tan();

        let a = z_near.abs() * tan_half_fov;
        let b = z_far.abs() * tan_half_fov;
        let aspect_ratio = self.aspect_ratio;

        let near_bottom_right = Vec3A::new(a * aspect_ratio, -a, z_near);
        let near_top_right = Vec3A::new(a * aspect_ratio, a, z_near);
        let near_top_left = Vec3A::new(-a * aspect_ratio, a, z_near);
        let near_bottom_left = Vec3A::new(-a * aspect_ratio, -a, z_near);

        let far_bottom_right = Vec3A::new(b * aspect_ratio, -b, z_far);
        let far_top_right = Vec3A::new(b * aspect_ratio, b, z_far);
        let far_top_left = Vec3A::new(-b * aspect_ratio, b, z_far);
        let far_bottom_left = Vec3A::new(-b * aspect_ratio, -b, z_far);

        // NOTE: These vertices are in the specific order required by [`calculate_cascade`].
        [
            near_bottom_right,
            near_top_right,
            near_top_left,
            near_bottom_left,
            far_bottom_right,
            far_top_right,
            far_top_left,
            far_bottom_left,
        ]
    }
}

impl Default for PortalPerspectiveProjection {
    fn default() -> Self {
        Self {
            fov: std::f32::consts::FRAC_PI_4,
            aspect_ratio: 1.0,
            near: 0.1,
            portal_frame: (
                0.0,
                Plane3d {
                    normal: Dir3::Z,
                    half_size: Vec2::new(0.5, 1.),
                },
            ),
            far: 1000.0,
        }
    }
}

pub struct PortalPlugin<P1: PortalKind<Pair = P2>, P2: PortalKind<Pair = P1>> {
    _p1: std::marker::PhantomData<P1>,
    _p2: std::marker::PhantomData<P2>,
}

impl<P1: PortalKind<Pair = P2>, P2: PortalKind<Pair = P1>> Default for PortalPlugin<P1, P2> {
    fn default() -> Self {
        Self {
            _p1: Default::default(),
            _p2: Default::default(),
        }
    }
}

impl<P1: PortalKind<Pair = P2>, P2: PortalKind<Pair = P1>> Plugin for PortalPlugin<P1, P2> {
    fn build(&self, app: &mut App) {
        use bevy::render::camera::CameraProjectionPlugin;

        app.register_portal_types()
            .add_event::<SpawnPortal<P1>>()
            .add_event::<SpawnPortal<P2>>()
            .add_systems(
                Update,
                (
                    (shoot_portal::<P1, P2>, remove_portals).in_set(input::ButtonInputReactions),
                    debug_info::portal_surface_gizmo,
                    debug_info::portal_gizmo,
                ),
            )
            .observe(spawn_portal::<P1>)
            .observe(spawn_portal::<P2>);

        if !app.is_plugin_added::<MaterialPlugin<PortalViewMaterial>>() {
            app.add_plugins(MaterialPlugin::<PortalViewMaterial>::default());
        }
        if !app.is_plugin_added::<CameraProjectionPlugin<PortalPerspectiveProjection>>() {
            app.add_plugins(CameraProjectionPlugin::<PortalPerspectiveProjection>::default());
        }
        if !app.is_plugin_added::<PbrProjectionPlugin<PortalPerspectiveProjection>>() {
            app.add_plugins(PbrProjectionPlugin::<PortalPerspectiveProjection>::default());
        }
    }
}
