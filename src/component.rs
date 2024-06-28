use bevy::prelude::*;

use crate::ExpDecay;

#[derive(Component)]
pub struct Player;

#[derive(Clone, Copy, Component)]
pub struct Velocity(pub Vec3);

impl Velocity {
    pub fn exp_decay_horizontal(&mut self, target: Vec2, decay: f32, delta: f32) {
        let Self(Vec3 { x, y: _, z }) = self;
        *x = x.exp_decay(target.x, decay, delta);
        *z = z.exp_decay(target.y, decay, delta);
    }
}

#[derive(Component)]
pub struct DebugInfoRoot;

#[derive(Component)]
pub struct DebugInfoText;
