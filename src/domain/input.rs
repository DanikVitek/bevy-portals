use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::resource::{Controls, ControlsConfig};

pub fn setup(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = window_query.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}

pub fn input_mappings(
    input: Res<ButtonInput<KeyCode>>,
    controls_config: Res<ControlsConfig>,
    mut controls: ResMut<Controls>,
) {
    controls.up = input.pressed(controls_config.up);
    controls.down = input.pressed(controls_config.down);
    controls.left = input.pressed(controls_config.left);
    controls.right = input.pressed(controls_config.right);
    controls.run = input.pressed(controls_config.run);
    controls.jump = input.just_pressed(controls_config.jump);
}

pub fn cursor_grab(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    input: Res<ButtonInput<MouseButton>>,
) {
    if input.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        let mut primary_window = window_query.single_mut();
        primary_window.cursor.grab_mode = CursorGrabMode::Locked;
        primary_window.cursor.visible = false;
    }
}

pub fn cursor_ungrab(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        let mut primary_window = window_query.single_mut();
        primary_window.cursor.grab_mode = CursorGrabMode::None;
        primary_window.cursor.visible = true;
    }
}
