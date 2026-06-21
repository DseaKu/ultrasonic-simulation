use bevy::prelude::*;

mod component;
mod constant;
mod system;

pub struct UltrasonicSensor;
impl Plugin for UltrasonicSensor {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, system::setup_sensor);
    }
}
