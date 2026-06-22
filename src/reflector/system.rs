use avian2d::prelude::{Collider, RigidBody};
use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;

use super::component;

pub fn setup_reflector(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: BLACK.into(),
            custom_size: Some(Vec2::new(40.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(200.0, 0.0, 0.0),
        Collider::rectangle(40.0, 40.0),
        RigidBody::Kinematic,
        component::Reflector::new(),
    ));
}
