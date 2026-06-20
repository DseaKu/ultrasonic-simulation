use bevy::prelude::*;

mod simulator;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(simulator::Simulator)
        .run();
}
