pub mod domain;
pub mod resource;

use bevy::{prelude::*, render::view::RenderLayers};
use cfg_if::cfg_if;
use once_cell::sync::Lazy;

pub static ALL_RENDER_LAYERS: Lazy<RenderLayers> = Lazy::new(|| {
    #[cfg(feature = "debug")]
    use bevy_editor_pls::default_windows::cameras::EDITOR_RENDER_LAYER;
    use domain::{player::PLAYER_RENDER_LAYER, scene::GROUND_RENDER_LAYER};

    let rl = RenderLayers::none()
        .with(GROUND_RENDER_LAYER)
        .with(PLAYER_RENDER_LAYER);

    cfg_if! {
        if #[cfg(feature = "debug")] {
            rl.with(EDITOR_RENDER_LAYER)
        } else {
            rl
        }
    }
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

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
