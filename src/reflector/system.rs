use bevy::prelude::*;

use super::bundle;

pub fn setup_reflector(mut commands: Commands) {
    commands.spawn(bundle::ReflectorBundle::new());
}

