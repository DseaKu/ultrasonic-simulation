use super::bundle;
use super::component;
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn setup_sensor(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(bundle::UltrasonicSensorBundle::new(asset_server));
}

pub fn collect_sensor_data(
    spatial_query: SpatialQuery,
    time: Res<Time>,
    mut query: Query<(
        &Transform,
        &component::UltrasonicSensor,
        &mut component::SensorHits,
        &mut component::HitHistory,
    )>,
    mut gizmos: Gizmos,
) {
    let dt = time.delta_secs();

    for (transform, sensor, mut sensor_hits, mut hit_history) in query.iter_mut() {
        sensor_hits.hits.clear();
        let mut next_history = std::collections::HashMap::new();

        let origin = transform.translation.xy();
        let forward_vec3 = transform.rotation * Vec3::X;
        let forward = Vec2::new(forward_vec3.x, forward_vec3.y).normalize_or_zero();

        let ray_count = sensor.ray_count;
        let beam_angle = sensor.beam_angle;
        let c = sensor.speed_of_sound;
        let f_t = sensor.frequency;

        let hit_color = Color::srgb(0.0, 1.0, 1.0); // Cyan
        let miss_color = Color::srgb(0.5, 0.5, 0.5); // Gray

        for i in 0..ray_count {
            let angle_offset = if ray_count > 1 {
                let t = i as f32 / (ray_count - 1) as f32;
                (t - 0.5) * beam_angle
            } else {
                0.0
            };

            // Rotate the forward vector by angle_offset
            let ray_direction = Vec2::new(
                forward.x * angle_offset.cos() - forward.y * angle_offset.sin(),
                forward.x * angle_offset.sin() + forward.y * angle_offset.cos(),
            );

            let ray_dir2 = Dir2::new(ray_direction).unwrap_or(Dir2::X);

            if let Some(hit) = spatial_query.cast_ray(
                origin,
                ray_dir2,
                sensor.max_range,
                true,
                &SpatialQueryFilter::default(),
            ) {
                let hit_point = origin + ray_direction * hit.distance;
                let d_current = hit.distance;

                // 1. Relative Velocity (v)
                let mut v = 0.0;
                if dt > 0.0 {
                    if let Some(&d_previous) = hit_history.distances.get(&(hit.entity, i)) {
                        v = (d_current - d_previous) / dt;
                    }
                }

                // 2. Time of Flight (t_delay)
                let delay = (2.0 * d_current) / c;

                // 3. Two-Way Doppler Shift (f_r)
                let denom = c + v;
                let doppler_freq = if denom.abs() > 0.001 {
                    f_t * ((c - v) / denom)
                } else {
                    f_t
                };

                sensor_hits.hits.push(component::RayHit {
                    entity: hit.entity,
                    point: hit_point,
                    distance: d_current,
                    velocity: v,
                    delay,
                    doppler_freq,
                });

                next_history.insert((hit.entity, i), d_current);

                // Draw a cyan line to the hit point
                gizmos.line_2d(origin, hit_point, hit_color);
            } else {
                // Draw a dim gray line representing the ray's path in the air
                gizmos.line_2d(
                    origin,
                    origin + ray_direction * sensor.max_range,
                    miss_color,
                );
            }
        }

        // Store history for the next frame
        hit_history.distances = next_history;
    }
}

pub fn synthesize_signal(
    mut query: Query<(
        &component::UltrasonicSensor,
        &component::SensorHits,
        &mut component::UltrasonicSignal,
    )>,
) {
    for (sensor, sensor_hits, mut ultrasonic_signal) in query.iter_mut() {
        let fs = 200_000.0; // 200 kHz sample rate
        let dt_s = 1.0 / fs;
        let t_max = 2.0 * sensor.max_range / sensor.speed_of_sound;
        let num_samples = (t_max / dt_s).ceil() as usize;

        let mut signal = vec![0.0; num_samples];
        let mut time_axis = vec![0.0; num_samples];
        for j in 0..num_samples {
            time_axis[j] = j as f32 * dt_s;
        }

        let sigma = sensor.pulse_width;
        let sigma_sq = sigma * sigma;

        for hit in sensor_hits.hits.iter() {
            let t_d = hit.delay;
            let f_r = hit.doppler_freq;

            // Sparse evaluation: within +/- 4 sigma
            let t_start = (t_d - 4.0 * sigma).max(0.0);
            let t_end = (t_d + 4.0 * sigma).min(t_max);

            let idx_start = (t_start / dt_s) as usize;
            let idx_end = ((t_end / dt_s) as usize).min(num_samples - 1);

            for j in idx_start..=idx_end {
                let t = time_axis[j];
                let diff = t - t_d;
                let env = (-diff * diff / (2.0 * sigma_sq)).exp();
                let wave = env * (2.0 * std::f32::consts::PI * f_r * diff).cos();
                signal[j] += wave;
            }
        }

        ultrasonic_signal.time_axis = time_axis;
        ultrasonic_signal.signal = signal;
        // Keep envelope vector matching in size (to be populated in step 5)
        ultrasonic_signal.envelope = vec![0.0; num_samples];
    }
}

