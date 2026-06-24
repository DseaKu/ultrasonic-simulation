use bevy::prelude::*;

mod bundle;
mod component;
mod constant;
mod system;

pub struct UltrasonicSensorPlugin;
impl Plugin for UltrasonicSensorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (system::setup_sensor, system::setup_time_scale))
            .add_systems(
                Update,
                (
                    system::adjust_time_scale,
                    system::adjust_sensor_gain,
                    system::adjust_doppler_exaggeration,
                    system::collect_sensor_data,
                    system::synthesize_signal,
                    system::plot_sensor_signal,
                ).chain(),
            );
    }
}
