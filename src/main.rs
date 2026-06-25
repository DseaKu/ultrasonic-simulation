use avian2d::prelude::*;
use bevy::prelude::*;

mod camera;
mod reflector;
mod ultrasonic_sensor;

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins);
    
    bevy::asset::embedded_asset!(app, "src/", "../assets/sensor.png");

    app.add_plugins(bevy_egui::EguiPlugin::default())
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec2::ZERO))
        .add_plugins(ultrasonic_sensor::UltrasonicSensorPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(reflector::ReflectorPlugin)
        .run();
}

