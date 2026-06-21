use super::component;
use bevy::prelude::*;

pub fn setup_sensor(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        component::UltrasonicSensor::new(),
        Sprite::from_image(asset_server.load("sensor.png")),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
