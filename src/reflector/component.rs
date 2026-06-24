use bevy::prelude::*;

use super::constant;

#[derive(Component)]
pub struct Reflector {
    pub speed: f32,
    pub current_velocity: Vec2,
}
impl Reflector {
    pub fn new() -> Self {
        Self {
            speed: constant::SPEED,
            current_velocity: Vec2::ZERO,
        }
    }
}
