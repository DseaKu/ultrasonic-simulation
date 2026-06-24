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
                custom_size: Some(Vec2::new(120.0, 120.0)),
                ..default()
            },
            transform: Transform::from_xyz(200.0, 0.0, 0.0),
            collider: Collider::rectangle(120.0, 120.0),
            rigid_body: RigidBody::Kinematic,
            reflector: component::Reflector::new(),
        }
    }
}
