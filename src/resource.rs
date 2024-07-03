use bevy::prelude::*;

#[derive(Resource)]
pub struct Pause(pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Control {
    KeyCode(KeyCode),
    MouseButton(MouseButton),
}

impl Control {
    pub fn pressed(
        self,
        input_key_code: &ButtonInput<KeyCode>,
        input_mouse_button: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Control::KeyCode(key_code) => input_key_code.pressed(key_code),
            Control::MouseButton(mouse_button) => input_mouse_button.pressed(mouse_button),
        }
    }

    pub fn just_pressed(
        self,
        input_key_code: &ButtonInput<KeyCode>,
        input_mouse_button: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Control::KeyCode(key_code) => input_key_code.just_pressed(key_code),
            Control::MouseButton(mouse_button) => input_mouse_button.just_pressed(mouse_button),
        }
    }
}

#[derive(Debug, Resource)]
pub struct ControlsConfig {
    pub up: Control,
    pub down: Control,
    pub left: Control,
    pub right: Control,
    pub run: Control,
    pub jump: Control,
    pub shoot1: Control,
    pub shoot2: Control,
}

impl Default for ControlsConfig {
    fn default() -> Self {
        Self {
            up: Control::KeyCode(KeyCode::KeyW),
            down: Control::KeyCode(KeyCode::KeyS),
            left: Control::KeyCode(KeyCode::KeyA),
            right: Control::KeyCode(KeyCode::KeyD),
            run: Control::KeyCode(KeyCode::ControlLeft),
            jump: Control::KeyCode(KeyCode::Space),
            shoot1: Control::MouseButton(MouseButton::Left),
            shoot2: Control::MouseButton(MouseButton::Right),
        }
    }
}

#[derive(Debug, Resource)]
pub struct MouseSensitivity(pub f32);

impl Default for MouseSensitivity {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Debug, Clone, Copy, Default, Resource)]
pub struct Controls {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub run: bool,
    pub jump: bool,
    pub shoot1: bool,
    pub shoot2: bool,
}

impl Controls {
    pub fn to_direction(self) -> Vec2 {
        Vec2::new(
            if self.right { 1.0 } else { 0.0 } - if self.left { 1.0 } else { 0.0 },
            if self.down { 1.0 } else { 0.0 } - if self.up { 1.0 } else { 0.0 },
        )
        .normalize_or_zero()
    }
}
