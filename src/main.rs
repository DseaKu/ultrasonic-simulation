use bevy::prelude::*;

mod camera;
mod reflector;
mod ultrasonic_sensor;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(ultrasonic_sensor::UltrasonicSensorPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(reflector::ReflectorPlugin)
        .run();
}
