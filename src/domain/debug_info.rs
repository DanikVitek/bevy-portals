use bevy::prelude::*;

use super::{
    player::{Grounded, Player},
    portal::{Portal1, Portal2, PortalSurface},
};

#[derive(Component)]
pub struct DebugInfoRoot;

#[derive(Component)]
pub struct DebugInfoText;

#[derive(Component)]
pub struct TranslationFromOriginGizmo;

pub fn translation_from_origin_gizmo(
    q: Query<(&GlobalTransform, &TranslationFromOriginGizmo)>,
    mut gizmos: Gizmos,
) {
    for (transform, _) in q.iter() {
        gizmos.line(Vec3::ZERO, transform.translation(), Color::WHITE);
    }
}

pub fn setup(mut commands: Commands) {
    commands
        .spawn((
            DebugInfoRoot,
            NodeBundle {
                z_index: ZIndex::Global(i32::MAX),
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(1.),
                    top: Val::Percent(1.),
                    bottom: Val::Auto,
                    left: Val::Auto,
                    padding: UiRect::all(Val::Px(4.)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .with_children(|child| {
            child.spawn((
                DebugInfoText,
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new("Player is grounded: ", TextStyle::default()),
                        TextSection::new("N/A", TextStyle::default()),
                        TextSection::new("\n", TextStyle::default()),
                    ]),
                    ..Default::default()
                },
            ));
        });
}

pub fn player_is_grounded(
    player_q: Query<&Grounded, With<Player>>,
    mut debug_info_q: Query<&mut Text, With<DebugInfoText>>,
) {
    let Ok(Grounded(grounded)) = player_q.get_single() else {
        return;
    };
    let mut debug_info_text = debug_info_q.single_mut();
    debug_info_text.sections[1].value = grounded.to_string();
}

pub fn portal_surface_gizmo(
    portal_surface_q: Query<(&GlobalTransform, &PortalSurface)>,
    mut gizmos: Gizmos,
) {
    for (transform, PortalSurface { size }) in portal_surface_q.iter() {
        let (scale, rotation, translation) = transform.to_scale_rotation_translation();
        gizmos.rect(translation, rotation, *size * scale.xy(), Color::WHITE);
        gizmos.ray(translation, transform.back(), Color::WHITE);
    }
}

pub fn portal_gizmo(
    portal1_q: Query<&GlobalTransform, With<Portal1>>,
    portal2_q: Query<&GlobalTransform, With<Portal2>>,
    mut gizmos: Gizmos,
) {
    for transform in portal1_q.iter() {
        let (scale, rotation, translation) = transform.to_scale_rotation_translation();
        gizmos.rect(
            translation,
            rotation,
            Vec2::new(1., 2.) * scale.xy(),
            Color::BLUE,
        );
        gizmos.ray(translation, transform.back(), Color::BLUE);
    }
    for transform in portal2_q.iter() {
        let (scale, rotation, translation) = transform.to_scale_rotation_translation();
        gizmos.rect(
            translation,
            rotation,
            Vec2::new(1., 2.) * scale.xy(),
            Color::ORANGE,
        );
        gizmos.ray(translation, transform.back(), Color::ORANGE);
    }
}
