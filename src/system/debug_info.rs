use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::component::{DebugInfoText, Player};

pub fn player_is_grounded(
    player_query: Query<&KinematicCharacterControllerOutput, With<Player>>,
    mut debug_info_query: Query<&mut Text, With<DebugInfoText>>,
) {
    let Ok(controller_output) = player_query.get_single() else {
        return;
    };
    let mut debug_info_text = debug_info_query.single_mut();
    debug_info_text.sections[1].value = controller_output.grounded.to_string();
}
