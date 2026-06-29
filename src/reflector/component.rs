use bevy::prelude::*;

use super::constant;

#[derive(Component)]
pub struct Reflector {
    pub speed: f32,
    pub current_velocity: Vec2,
    pub width: f32,
    pub height: f32,
    pub spin: f32,
}
impl Reflector {
    pub fn new() -> Self {
        Self {
            speed: constant::defaults::SPEED,
            current_velocity: Vec2::ZERO,
            width: constant::dimensions::WIDTH,
            height: constant::dimensions::HEIGHT,
            spin: 0.0,
        }
    }
}
