use bevy::prelude::*;

use super::bundle;

pub fn setup_reflector(mut commands: Commands) {
    commands.spawn((bundle::ReflectorBundle::new(), super::component::SelectedReflector));
}

pub fn move_reflector(
    time: Res<Time<Virtual>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Has<super::component::SelectedReflector>, &mut Transform, &mut super::component::Reflector)>,
) {
    let dt = time.delta_secs();

    for (is_selected, mut transform, mut reflector) in query.iter_mut() {
        let mut input_direction = Vec2::ZERO;

        if is_selected {
            // X-axis: A/D keys
            if keyboard.pressed(KeyCode::KeyA) {
                input_direction.x -= 1.0;
            }
            if keyboard.pressed(KeyCode::KeyD) {
                input_direction.x += 1.0;
            }

            // Y-axis: W/S keys
            if keyboard.pressed(KeyCode::KeyW) {
                input_direction.y += 1.0;
            }
            if keyboard.pressed(KeyCode::KeyS) {
                input_direction.y -= 1.0;
            }
        }

        let speed_mm_s = reflector.speed * 1000.0;
        let target_velocity = input_direction.normalize_or_zero() * speed_mm_s;

        // Smoothly interpolate current velocity towards target velocity
        // Higher lerp factor = faster acceleration/deceleration
        reflector.current_velocity = reflector.current_velocity.lerp(
            target_velocity,
            (super::constant::defaults::ACCELERATION_FACTOR * dt).min(1.0),
        );

        transform.translation += reflector.current_velocity.extend(0.0) * dt;

        // Clamp coordinates to keep reflector within visible and physical bounds
        transform.translation.x = transform.translation.x.clamp(
            super::constant::bounds::MIN_X,
            super::constant::bounds::MAX_X,
        );
        transform.translation.y = transform.translation.y.clamp(
            super::constant::bounds::MIN_Y,
            super::constant::bounds::MAX_Y,
        );

        // Apply spin (continuous rotation)
        transform.rotate_z(reflector.spin * dt);
    }
}

pub fn update_reflector_dimensions(
    mut commands: Commands,
    mut query: Query<(Entity, &super::component::Reflector, &mut Sprite), Changed<super::component::Reflector>>,
) {
    for (entity, reflector, mut sprite) in query.iter_mut() {
        let new_size = Vec2::new(reflector.width, reflector.height);
        if sprite.custom_size != Some(new_size) {
            sprite.custom_size = Some(new_size);
            commands.entity(entity).insert(avian2d::prelude::Collider::rectangle(reflector.width, reflector.height));
        }
    }
}
