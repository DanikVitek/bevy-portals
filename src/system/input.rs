use bevy::prelude::*;

use crate::resource::{Controls, ControlsConfig};

pub fn input_mappings(
    input: Res<ButtonInput<KeyCode>>,
    controls_config: Res<ControlsConfig>,
    mut controls: ResMut<Controls>,
) {
    controls.up = input.pressed(controls_config.up);
    controls.down = input.pressed(controls_config.down);
    controls.left = input.pressed(controls_config.left);
    controls.right = input.pressed(controls_config.right);
}