use bevy::prelude::*;

use super::bundle;

pub fn setup_reflector(mut commands: Commands) {
    commands.spawn(bundle::ReflectorBundle::new());
}

pub fn move_reflector(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &super::component::Reflector)>,
) {
    let dt = time.delta_secs();

    for (mut transform, reflector) in query.iter_mut() {
        let speed = reflector.speed;
        let mut moved = false;

        // Manual controls: Left/Right Arrow or A/D keys
        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            transform.translation.x -= speed * dt;
            moved = true;
        }
        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            transform.translation.x += speed * dt;
            moved = true;
        }

        // Clamp within the viewport / simulation bounds (the sensor is at x = -400.0)
        transform.translation.x = transform.translation.x.clamp(-250.0, 350.0);

        // Auto mode: If no manual input, oscillate back and forth
        if !moved {
            let t = time.elapsed_secs();
            // Oscillate X between -150 and 250
            let amplitude = 200.0;
            let center = 50.0;
            let frequency = 0.15; // 0.15 Hz oscillation frequency
            transform.translation.x = center + (t * frequency * 2.0 * std::f32::consts::PI).sin() * amplitude;
        }
    }
}


