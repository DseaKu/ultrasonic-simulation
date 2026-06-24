use bevy::prelude::*;

use super::bundle;

pub fn setup_reflector(mut commands: Commands) {
    commands.spawn(bundle::ReflectorBundle::new());
}

pub fn move_reflector(
    time: Res<Time<Virtual>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &super::component::Reflector)>,
) {
    let dt = time.delta_secs();

    for (mut transform, reflector) in query.iter_mut() {
        let speed = reflector.speed;

        // X-axis: Left/Right Arrow or A/D keys
        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            transform.translation.x -= speed * dt;
        }
        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            transform.translation.x += speed * dt;
        }

        // Y-axis: Up/Down Arrow or W/S keys
        if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
            transform.translation.y += speed * dt;
        }
        if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
            transform.translation.y -= speed * dt;
        }

        // Clamp coordinates to keep reflector within visible and physical bounds
        transform.translation.x = transform.translation.x.clamp(-250.0, 350.0);
        transform.translation.y = transform.translation.y.clamp(-220.0, 220.0);
    }
}


