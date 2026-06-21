use bevy::prelude::*;

mod component;
mod constant;
mod system;

pub struct UltrasonicSensorPlugin;
impl Plugin for UltrasonicSensorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, system::setup_sensor);
    }
}
