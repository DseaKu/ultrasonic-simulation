use bevy::prelude::*;

mod bundle;
mod component;
mod constant;
mod system;

pub struct UltrasonicSensorPlugin;
impl Plugin for UltrasonicSensorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (system::setup_sensor, system::setup_ui))
            .add_systems(
                Update,
                (
                    system::toggle_pause,
                    system::handle_ui_interactions,
                    system::update_ui_text,
                    system::toggle_rays,
                    system::collect_sensor_data,
                    system::synthesize_signal,
                    system::plot_sensor_signal,
                ).chain(),
            );
    }
}
