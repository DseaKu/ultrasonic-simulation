use super::component;
use bevy::prelude::*;

use super::constant;

pub fn setup_sensor(mut commands: Commands, asset_server: Res<AssetServer>) {
    use constant::spawn_position as pos;
    commands.spawn((
        component::UltrasonicSensor::new(),
        Sprite::from_image(asset_server.load("sensor.png")),
        Transform::from_xyz(pos::X, pos::Y, pos::Z),
    ));
}
