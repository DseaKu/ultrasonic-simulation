use avian2d::prelude::*;
use bevy::prelude::*;

mod camera;
mod reflector;
mod ultrasonic_sensor;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_egui::EguiPlugin::default())
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec2::ZERO))
        .add_plugins(ultrasonic_sensor::UltrasonicSensorPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(reflector::ReflectorPlugin)
        .run();
}

