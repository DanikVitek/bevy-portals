use bevy::{ecs::query::QuerySingleError, prelude::*};
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
pub struct Portal {
    pair: Option<Entity>,
}

pub trait PortalKind: Component {
    type Pair: PortalKind<Pair = Self>;

    fn color() -> Color;
}

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Portal1;

#[derive(Debug, Default, Component, Reflect)]
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

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn shoot_portal(
    controls: Res<Controls>,
    camera_q: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
    portal_surface_q: Query<(&GlobalTransform, &PortalSurface)>,
    mut portal1_q: Query<(Entity, &mut Portal), (With<Portal1>, Without<Portal2>)>,
    mut portal2_q: Query<(Entity, &mut Portal), (With<Portal2>, Without<Portal1>)>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    rapier_ctx: Res<RapierContext>,
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
        println!("Spawning portal 1");
        match portal1_q.get_single() {
            Ok((portal1, _)) => commands.entity(portal1).despawn(),
            Err(err @ QuerySingleError::MultipleEntities(_)) => unreachable!("{err}"),
            Err(QuerySingleError::NoEntities(_)) => {}
        }
        spawn_portal(
            Portal1,
            portal_transform,
            portal2_q.iter_mut().next(),
            commands,
            meshes,
            materials,
        );
    } else if controls.shoot2 {
        println!("Spawning portal 2");
        match portal2_q.get_single() {
            Ok((portal2, _)) => commands.entity(portal2).despawn(),
            Err(err @ QuerySingleError::MultipleEntities(_)) => unreachable!("{err}"),
            Err(QuerySingleError::NoEntities(_)) => {}
        }
        spawn_portal(
            Portal2,
            portal_transform,
            portal1_q.iter_mut().next(),
            commands,
            meshes,
            materials,
        );
    }
}

fn spawn_portal<P: PortalKind>(
    portal: P,
    transform: Transform,
    pair: Option<(Entity, Mut<Portal>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let new_portal = commands
        .spawn((
            Portal {
                pair: pair.iter().map(|(entity, _)| entity).copied().next(),
            },
            portal,
            PbrBundle {
                mesh: meshes.add(Plane3d::new(Vec3::Z).mesh().size(1., 2.)),
                material: materials.add(P::color()),
                transform,
                ..Default::default()
            },
        ))
        .id();
    if let Some(Portal { pair }) = pair.map(|(_, portal)| portal).as_deref_mut() {
        _ = pair.replace(new_portal);
    }
}
