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
    pub remove_portals: Control,
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
            remove_portals: Control::KeyCode(KeyCode::KeyR),
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

#[derive(Debug, Clone, Copy, Resource)]
pub struct FieldOfView(f32);

impl FieldOfView {
    pub const DEG60: Self = Self(std::f32::consts::FRAC_PI_3);
    pub const DEG90: Self = Self(std::f32::consts::FRAC_PI_2);
    pub const DEG180: Self = Self(std::f32::consts::PI);

    pub fn from_degrees(degrees: f32) -> Self {
        Self(degrees.to_radians())
    }

    pub fn from_radians(radians: f32) -> Self {
        Self(radians)
    }

    pub fn degrees(&self) -> f32 {
        self.0.to_degrees()
    }

    pub fn radians(&self) -> f32 {
        self.0
    }
}

impl Default for FieldOfView {
    fn default() -> Self {
        Self::DEG90
    }
}

pub type Fov = FieldOfView;

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
    pub remove_portals: bool,
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
