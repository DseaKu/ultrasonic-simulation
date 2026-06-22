use super::component;
use bevy::prelude::*;

use super::constant;

#[derive(Bundle)]
pub struct UltrasonicSensorBundle {
    ultrasonic_sensor_component: component::UltrasonicSensor,
    hit_history: component::HitHistory,
    sensor_hits: component::SensorHits,
    ultrasonic_signal: component::UltrasonicSignal,
    sprite: Sprite,
    transform: Transform,
}

impl UltrasonicSensorBundle {
    pub fn new(asset_server: Res<AssetServer>) -> Self {
        use constant::spawn_position as pos;
        Self {
            ultrasonic_sensor_component: component::UltrasonicSensor::new(),
            hit_history: component::HitHistory::new(),
            sensor_hits: component::SensorHits::new(),
            ultrasonic_signal: component::UltrasonicSignal::new(),
            sprite: Sprite::from_image(asset_server.load("sensor.png")),
            transform: Transform::from_xyz(pos::X, pos::Y, pos::Z),
        }
    }
}
