pub mod component;
pub mod resource;
pub mod system;

use bevy::prelude::*;

pub const PLAYER_RENDER_LAYER: u8 = 1;

pub trait ExpDecay {
    fn exp_decay(self, rhs: Self, decay: Self, dt: f32) -> Self;
}

impl ExpDecay for f32 {
    fn exp_decay(self, rhs: Self, decay: Self, dt: f32) -> Self {
        rhs + (self - rhs) * (-decay * dt).exp()
    }
}

impl ExpDecay for Vec2 {
    fn exp_decay(
        self,        // m/s
        rhs: Self,   // m/s
        decay: Self, // 1/s
        dt: f32,     // s
    ) -> Self {
        rhs + (self - rhs) * (-decay * dt).exp()
    }
}

impl ExpDecay for Vec3 {
    fn exp_decay(self, rhs: Self, decay: Self, dt: f32) -> Self {
        rhs + (self - rhs) * (-decay * dt).exp()
    }
}
