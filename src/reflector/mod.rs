use bevy::prelude::*;

mod bundle;
mod component;
pub mod constant;
mod system;

pub struct ReflectorPlugin;
impl Plugin for ReflectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, system::setup_reflector)
            .add_systems(Update, system::move_reflector);
    }
}

