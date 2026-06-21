use bevy::prelude::*;

mod ultrasonic_sensor;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ultrasonic_sensor::UltrasonicSensor)
        .run();
}
