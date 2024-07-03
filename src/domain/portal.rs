use bevy::{ecs::query::QuerySingleError, prelude::*};
use itertools::Itertools;

use crate::resource::Controls;

use super::player::PlayerCamera;

pub const DEFAULT_PORTAL_SIZE: Vec2 = Vec2::new(1., 2.);

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

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Portal1;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Portal2;

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
) {
    if !controls.shoot1 && !controls.shoot2 {
        return;
    }

    println!("Shooting portal\n");

    let (camera, camera_transform) = camera_q.single();
    let viewport_center = camera.logical_viewport_size().unwrap() / 2.;

    let Some(ray) = camera.viewport_to_world(camera_transform, viewport_center) else {
        return;
    };

    println!("Got camera ray: {ray:?}\n");

    let Some(portal_transform) = portal_surface_q
        .iter()
        .filter_map(|(transform, &PortalSurface { size })| {
            ray.intersect_plane(transform.translation(), Plane3d::new(transform.forward()))
                .inspect(|distance| {
                    println!("Intersect with plane {transform:?} at distance {distance}\n")
                })
                .and_then(|distance| {
                    let point = ray.get_point(distance);
                    println!("Intersect with plane {transform:?} at point {point}\n");
                    let point_on_plane = transform.affine().inverse().transform_point3(point);
                    println!("Point on the plane: {point_on_plane}\n");

                    let half_size = size * transform.to_scale_rotation_translation().0.xy() / 2.;

                    (point_on_plane.x.abs() < half_size.x && point_on_plane.y.abs() < half_size.y)
                        .then(|| {
                            let mut portal_transform = transform.compute_transform();
                            let clamped_point =
                                transform.affine().transform_point3(point_on_plane.clamp(
                                    (-half_size + DEFAULT_PORTAL_SIZE / 2.).extend(0.),
                                    (half_size - DEFAULT_PORTAL_SIZE / 2.).extend(0.),
                                ));
                            portal_transform.translation +=
                                (clamped_point - transform.translation()) + transform.back() * 0.01;
                            (distance, portal_transform)
                        })
                })
        })
        .sorted_unstable_by(|(distance1, _), (distance2, _)| distance1.total_cmp(distance2))
        .map(|(_, t)| t)
        .next()
    else {
        return;
    };

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

fn spawn_portal<P: Component>(
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
                material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
                transform,
                ..Default::default()
            },
        ))
        .id();
    if let Some(Portal { pair }) = pair.map(|(_, portal)| portal).as_deref_mut() {
        _ = pair.replace(new_portal);
    }
}
