use bevy::prelude::*;

use crate::ExpDecay;

#[derive(Clone, Copy, Component)]
pub struct Player;

#[derive(Clone, Copy, Component)]
pub struct HorizontalVelocity(pub Vec2);

impl HorizontalVelocity {
    pub fn exp_decay(self, target: Vec2, decay: f32, delta: f32) -> Self {
        Self(self.0.exp_decay(target, Vec2::splat(decay), delta))
    }
}
