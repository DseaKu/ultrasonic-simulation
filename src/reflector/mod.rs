use bevy::prelude::*;

mod component;
mod constant;
mod system;

pub struct ReflectorPlugin;
impl Plugin for ReflectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, system::setup_reflector);
    }
}
