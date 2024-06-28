use bevy::prelude::*;

#[derive(Resource)]
pub struct Pause(pub bool);

#[derive(Resource)]
pub struct ControlsConfig {
    pub up: KeyCode,
    pub down: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub run: KeyCode,
    pub jump: KeyCode,
}

impl Default for ControlsConfig {
    fn default() -> Self {
        Self {
            up: KeyCode::KeyW,
            down: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            run: KeyCode::ControlLeft,
            jump: KeyCode::Space,
        }
    }
}

#[derive(Resource)]
pub struct MouseSensitivity(pub f32);

impl Default for MouseSensitivity {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Debug, Clone, Copy, Resource, Default)]
pub struct Controls {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub run: bool,
    pub jump: bool,
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
