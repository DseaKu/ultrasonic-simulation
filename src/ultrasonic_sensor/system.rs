use super::component;
use bevy::prelude::*;

pub fn setup_sensor(mut commands: Commands) {
    commands.spawn((
        component::UltrasonicSenosr::new(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));
}
