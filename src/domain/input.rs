use bevy::{
    app::AppExit,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowCloseRequested},
};

use crate::resource::{Controls, ControlsConfig};

pub fn setup(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut exit: EventWriter<AppExit>,
) {
    let Ok(mut primary_window) = window_query.get_single_mut() else {
        exit.send(AppExit::Success);
        return;
    };
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}

pub fn input_mappings(
    input_key_code: Res<ButtonInput<KeyCode>>,
    input_mouse_button: Res<ButtonInput<MouseButton>>,
    controls_config: Res<ControlsConfig>,
    mut controls: ResMut<Controls>,
) {
    controls.up = controls_config
        .up
        .pressed(&input_key_code, &input_mouse_button);
    controls.down = controls_config
        .down
        .pressed(&input_key_code, &input_mouse_button);
    controls.left = controls_config
        .left
        .pressed(&input_key_code, &input_mouse_button);
    controls.right = controls_config
        .right
        .pressed(&input_key_code, &input_mouse_button);
    controls.run = controls_config
        .run
        .pressed(&input_key_code, &input_mouse_button);
    controls.jump = controls_config
        .jump
        .just_pressed(&input_key_code, &input_mouse_button);
    controls.shoot1 = controls_config
        .shoot1
        .just_pressed(&input_key_code, &input_mouse_button);
    controls.shoot2 = controls_config
        .shoot2
        .just_pressed(&input_key_code, &input_mouse_button);
    controls.remove_portals = controls_config
        .remove_portals
        .just_pressed(&input_key_code, &input_mouse_button);
}

pub fn exit_on_primary_close(
    mut close_events: EventReader<WindowCloseRequested>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
    mut exit: EventWriter<AppExit>,
) {
    for &WindowCloseRequested { window } in close_events.read() {
        if primary_window_q.get(window).is_ok() {
            exit.send(AppExit::Success);
            break;
        }
    }
}

pub fn cursor_grab(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    input: Res<ButtonInput<MouseButton>>,
    mut exit: EventWriter<AppExit>,
) {
    if input.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        let Ok(mut primary_window) = window_query.get_single_mut() else {
            exit.send(AppExit::Success);
            return;
        };
        primary_window.cursor.grab_mode = CursorGrabMode::Locked;
        primary_window.cursor.visible = false;
    }
}

pub fn cursor_ungrab(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if input.just_pressed(KeyCode::Escape) {
        let Ok(mut primary_window) = window_query.get_single_mut() else {
            exit.send(AppExit::Success);
            return;
        };
        primary_window.cursor.grab_mode = CursorGrabMode::None;
        primary_window.cursor.visible = true;
    }
}
