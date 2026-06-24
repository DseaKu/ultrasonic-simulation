use avian2d::prelude::{Collider, RigidBody};
use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;

use super::component;

#[derive(Bundle)]
pub struct ReflectorBundle {
    sprite: Sprite,
    transform: Transform,
    collider: Collider,
    rigid_body: RigidBody,
    reflector: component::Reflector,
}
impl ReflectorBundle {
    pub fn new() -> Self {
        Self {
            sprite: Sprite {
                color: BLACK.into(),
                custom_size: Some(Vec2::new(
                    super::constant::dimensions::WIDTH,
                    super::constant::dimensions::HEIGHT,
                )),
                ..default()
            },
            transform: Transform::from_xyz(
                super::constant::spawn_position::X,
                super::constant::spawn_position::Y,
                super::constant::spawn_position::Z,
            ),
            collider: Collider::rectangle(
                super::constant::dimensions::WIDTH,
                super::constant::dimensions::HEIGHT,
            ),
            rigid_body: RigidBody::Kinematic,
            reflector: component::Reflector::new(),
        }
    }
}
