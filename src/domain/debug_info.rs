use bevy::prelude::*;

use super::player::{Grounded, Player};

#[derive(Component)]
pub struct DebugInfoRoot;

#[derive(Component)]
pub struct DebugInfoText;

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
