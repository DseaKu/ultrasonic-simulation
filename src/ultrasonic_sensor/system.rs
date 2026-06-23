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
            let dist = hit.distance;

            // Physical distance attenuation: inverse square law (reference distance of 150.0) scaled by gain
            let atten = (150.0 / dist.max(150.0)).powi(2) * sensor.gain;

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
                signal[j] += wave * atten;
            }
        }

        // 1. Peak follower envelope detection
        let mut envelope = vec![0.0; num_samples];
        let mut env_val = 0.0;
        let decay = (-dt_s / 0.00015).exp(); // tau = 0.15 ms for 1 ms pulse width
        for j in 0..num_samples {
            let abs_val = signal[j].abs();
            if abs_val > env_val {
                env_val = abs_val;
            } else {
                env_val *= decay;
            }
            envelope[j] = env_val;
        }

        // Forward-backward smoothing for zero-phase alignment
        let mut smooth_envelope = vec![0.0; num_samples];
        let mut env_back = 0.0;
        for j in (0..num_samples).rev() {
            let val = envelope[j];
            if val > env_back {
                env_back = val;
            } else {
                env_back *= decay;
            }
            smooth_envelope[j] = (val + env_back) * 0.5;
        }

        ultrasonic_signal.time_axis = time_axis;
        ultrasonic_signal.signal = signal;
        ultrasonic_signal.envelope = smooth_envelope;
    }
}

// Draw a stroke sequence for our vector oscilloscope font
fn draw_stroke(gizmos: &mut Gizmos, points: &[(f32, f32)], pos: Vec2, size: f32, color: Color) {
    for i in 0..points.len().saturating_sub(1) {
        let p1 = pos + Vec2::new(points[i].0 * size, points[i].1 * size * 1.4);
        let p2 = pos + Vec2::new(points[i + 1].0 * size, points[i + 1].1 * size * 1.4);
        gizmos.line_2d(p1, p2, color);
    }
}

// Draw a single character in a clean vector/oscilloscope style
fn draw_char(gizmos: &mut Gizmos, c: char, pos: Vec2, size: f32, color: Color) {
    match c {
        '0' => {
            draw_stroke(
                gizmos,
                &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)],
                pos,
                size,
                color,
            );
            draw_stroke(gizmos, &[(0.0, 0.0), (1.0, 1.0)], pos, size, color);
        }
        '1' => {
            draw_stroke(
                gizmos,
                &[(0.5, 0.0), (0.5, 1.0), (0.3, 0.8)],
                pos,
                size,
                color,
            );
        }
        '2' => {
            draw_stroke(
                gizmos,
                &[
                    (0.0, 1.0),
                    (1.0, 1.0),
                    (1.0, 0.5),
                    (0.0, 0.5),
                    (0.0, 0.0),
                    (1.0, 0.0),
                ],
                pos,
                size,
                color,
            );
        }
        '3' => {
            draw_stroke(
                gizmos,
                &[(0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)],
                pos,
                size,
                color,
            );
            draw_stroke(gizmos, &[(0.0, 0.5), (1.0, 0.5)], pos, size, color);
        }
        '4' => {
            draw_stroke(
                gizmos,
                &[(0.0, 1.0), (0.0, 0.5), (1.0, 0.5)],
                pos,
                size,
                color,
            );
            draw_stroke(gizmos, &[(1.0, 1.0), (1.0, 0.0)], pos, size, color);
        }
        '5' => {
            draw_stroke(
                gizmos,
                &[
                    (1.0, 1.0),
                    (0.0, 1.0),
                    (0.0, 0.5),
                    (1.0, 0.5),
                    (1.0, 0.0),
                    (0.0, 0.0),
                ],
                pos,
                size,
                color,
            );
        }
        '6' => {
            draw_stroke(
                gizmos,
                &[
                    (1.0, 1.0),
                    (0.0, 1.0),
                    (0.0, 0.0),
                    (1.0, 0.0),
                    (1.0, 0.5),
                    (0.0, 0.5),
                ],
                pos,
                size,
                color,
            );
        }
        '7' => {
            draw_stroke(
                gizmos,
                &[(0.0, 1.0), (1.0, 1.0), (0.3, 0.0)],
                pos,
                size,
                color,
            );
        }
        '8' => {
            draw_stroke(
                gizmos,
                &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)],
                pos,
                size,
                color,
            );
            draw_stroke(gizmos, &[(0.0, 0.5), (1.0, 0.5)], pos, size, color);
        }
        '9' => {
            draw_stroke(
                gizmos,
                &[(1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.5), (1.0, 0.5)],
                pos,
                size,
                color,
            );
        }
        '-' => {
            draw_stroke(gizmos, &[(0.2, 0.5), (0.8, 0.5)], pos, size, color);
        }
        '.' => {
            draw_stroke(
                gizmos,
                &[(0.4, 0.0), (0.6, 0.0), (0.6, 0.2), (0.4, 0.2), (0.4, 0.0)],
                pos,
                size,
                color,
            );
        }
        ':' => {
            draw_stroke(
                gizmos,
                &[(0.4, 0.2), (0.6, 0.2), (0.6, 0.35), (0.4, 0.35), (0.4, 0.2)],
                pos,
                size,
                color,
            );
            draw_stroke(
                gizmos,
                &[
                    (0.4, 0.65),
                    (0.6, 0.65),
                    (0.6, 0.8),
                    (0.4, 0.8),
                    (0.4, 0.65),
                ],
                pos,
                size,
                color,
            );
        }
        'm' => {
            draw_stroke(
                gizmos,
                &[(0.0, 0.0), (0.0, 0.6), (0.5, 0.6), (0.5, 0.0)],
                pos,
                size,
                color,
            );
            draw_stroke(
                gizmos,
                &[(0.5, 0.6), (1.0, 0.6), (1.0, 0.0)],
                pos,
                size,
                color,
            );
        }
        's' => {
            draw_stroke(
                gizmos,
                &[
                    (1.0, 0.6),
                    (0.0, 0.6),
                    (0.0, 0.3),
                    (1.0, 0.3),
                    (1.0, 0.0),
                    (0.0, 0.0),
                ],
                pos,
                size,
                color,
            );
        }
        'd' => {
            draw_stroke(gizmos, &[(1.0, 0.8), (1.0, 0.0)], pos, size, color);
            draw_stroke(
                gizmos,
                &[(1.0, 0.4), (0.0, 0.4), (0.0, 0.0), (1.0, 0.0)],
                pos,
                size,
                color,
            );
        }
        't' => {
            draw_stroke(
                gizmos,
                &[(0.5, 0.8), (0.5, 0.0), (0.8, 0.0)],
                pos,
                size,
                color,
            );
            draw_stroke(gizmos, &[(0.2, 0.6), (0.8, 0.6)], pos, size, color);
        }
        'a' => {
            draw_stroke(gizmos, &[(1.0, 0.4), (1.0, 0.0)], pos, size, color);
            draw_stroke(
                gizmos,
                &[(1.0, 0.4), (0.0, 0.4), (0.0, 0.0), (1.0, 0.0)],
                pos,
                size,
                color,
            );
            draw_stroke(gizmos, &[(0.0, 0.4), (0.0, 0.2)], pos, size, color);
        }
        'n' => {
            draw_stroke(
                gizmos,
                &[(0.0, 0.0), (0.0, 0.5), (1.0, 0.5), (1.0, 0.0)],
                pos,
                size,
                color,
            );
        }
        'c' => {
            draw_stroke(
                gizmos,
                &[(1.0, 0.4), (0.0, 0.4), (0.0, 0.0), (1.0, 0.0)],
                pos,
                size,
                color,
            );
        }
        'e' => {
            draw_stroke(
                gizmos,
                &[
                    (0.0, 0.0),
                    (1.0, 0.0),
                    (1.0, 0.2),
                    (0.0, 0.2),
                    (0.0, 0.4),
                    (1.0, 0.4),
                ],
                pos,
                size,
                color,
            );
        }
        'v' => {
            draw_stroke(
                gizmos,
                &[(0.0, 0.5), (0.5, 0.0), (1.0, 0.5)],
                pos,
                size,
                color,
            );
        }
        'i' => {
            draw_stroke(gizmos, &[(0.5, 0.0), (0.5, 0.4)], pos, size, color);
            draw_stroke(gizmos, &[(0.5, 0.6), (0.5, 0.7)], pos, size, color);
        }
        'g' => {
            draw_stroke(
                gizmos,
                &[
                    (1.0, 0.4),
                    (0.0, 0.4),
                    (0.0, 0.0),
                    (1.0, 0.0),
                    (1.0, -0.4),
                    (0.0, -0.4),
                ],
                pos,
                size,
                color,
            );
        }
        'l' => {
            draw_stroke(gizmos, &[(0.5, 0.8), (0.5, 0.0)], pos, size, color);
        }
        'p' => {
            draw_stroke(
                gizmos,
                &[(0.0, -0.4), (0.0, 0.4), (1.0, 0.4), (1.0, 0.0), (0.0, 0.0)],
                pos,
                size,
                color,
            );
        }
        'u' => {
            draw_stroke(
                gizmos,
                &[(0.0, 0.4), (0.0, 0.0), (1.0, 0.0), (1.0, 0.4)],
                pos,
                size,
                color,
            );
        }
        'r' => {
            draw_stroke(
                gizmos,
                &[(0.0, 0.0), (0.0, 0.4), (1.0, 0.4)],
                pos,
                size,
                color,
            );
        }
        'S' => {
            draw_stroke(
                gizmos,
                &[
                    (1.0, 0.8),
                    (0.0, 0.8),
                    (0.0, 0.4),
                    (1.0, 0.4),
                    (1.0, 0.0),
                    (0.0, 0.0),
                ],
                pos,
                size,
                color,
            );
        }
        'D' => {
            draw_stroke(
                gizmos,
                &[
                    (0.0, 0.0),
                    (0.0, 1.0),
                    (0.7, 1.0),
                    (1.0, 0.7),
                    (1.0, 0.3),
                    (0.7, 0.0),
                    (0.0, 0.0),
                ],
                pos,
                size,
                color,
            );
        }
        'C' => {
            draw_stroke(
                gizmos,
                &[(1.0, 1.0), (0.0, 1.0), (0.0, 0.0), (1.0, 0.0)],
                pos,
                size,
                color,
            );
        }
        ' ' => {}
        _ => {
            draw_stroke(
                gizmos,
                &[(0.2, 0.2), (0.8, 0.2), (0.8, 0.8), (0.2, 0.8), (0.2, 0.2)],
                pos,
                size,
                color,
            );
        }
    }
}

// Draw a string using the vector font
fn draw_string(gizmos: &mut Gizmos, s: &str, start_pos: Vec2, size: f32, color: Color) {
    let mut current_pos = start_pos;
    for c in s.chars() {
        draw_char(gizmos, c, current_pos, size, color);
        current_pos.x += size * 1.3;
    }
}

// Plot system using Bevy Gizmos to visualize the synthetic signals
pub fn plot_sensor_signal(
    query: Query<(&component::UltrasonicSensor, &component::UltrasonicSignal)>,
    mut gizmos: Gizmos,
) {
    for (sensor, ultrasonic_signal) in query.iter() {
        if ultrasonic_signal.signal.is_empty() {
            continue;
        }

        // Define plotting region
        let plot_width = 1000.0;
        let plot_height = 160.0;
        let plot_center = Vec2::new(0.0, -250.0);

        let half_w = plot_width / 2.0;
        let half_h = plot_height / 2.0;

        let bottom_left = plot_center - Vec2::new(half_w, half_h);
        let top_right = plot_center + Vec2::new(half_w, half_h);

        let border_color = Color::srgb(0.4, 0.4, 0.4);
        let grid_color = Color::srgb(0.15, 0.15, 0.15);

        // Draw background frame using gizmos
        gizmos.line_2d(
            Vec2::new(bottom_left.x, bottom_left.y),
            Vec2::new(bottom_left.x, top_right.y),
            border_color,
        );
        gizmos.line_2d(
            Vec2::new(top_right.x, bottom_left.y),
            Vec2::new(top_right.x, top_right.y),
            border_color,
        );
        gizmos.line_2d(
            Vec2::new(bottom_left.x, bottom_left.y),
            Vec2::new(top_right.x, bottom_left.y),
            border_color,
        );
        gizmos.line_2d(
            Vec2::new(bottom_left.x, top_right.y),
            Vec2::new(top_right.x, top_right.y),
            border_color,
        );

        // Center line (zero amplitude)
        gizmos.line_2d(
            Vec2::new(bottom_left.x, plot_center.y),
            Vec2::new(top_right.x, plot_center.y),
            grid_color,
        );

        // Draw vertical grid ticks and labels (every 100 distance units)
        let max_range = sensor.max_range;
        let num_ticks = 8;
        for i in 0..=num_ticks {
            let t = i as f32 / num_ticks as f32;
            let x = bottom_left.x + t * plot_width;

            // Grid line
            if i > 0 && i < num_ticks {
                gizmos.line_2d(
                    Vec2::new(x, bottom_left.y),
                    Vec2::new(x, top_right.y),
                    grid_color,
                );
            }

            // Tick mark
            gizmos.line_2d(
                Vec2::new(x, bottom_left.y),
                Vec2::new(x, bottom_left.y - 5.0),
                border_color,
            );

            // Tick label (distance in mm/units)
            let dist = (t * max_range) as i32;
            let label = format!("{}", dist);
            draw_string(
                &mut gizmos,
                &label,
                Vec2::new(x - (label.len() as f32 * 3.5), bottom_left.y - 20.0),
                6.0,
                border_color,
            );
        }

        // Plot signal and envelope
        let signal = &ultrasonic_signal.signal;
        let envelope = &ultrasonic_signal.envelope;
        let num_samples = signal.len();

        // Use a fixed scaling factor based on the number of rays to visualize physical attenuation
        let max_possible_amp = sensor.ray_count as f32 * 0.45; // 45% of maximum theoretically possible sum when in-phase at close range
        let scale_y = half_h / max_possible_amp.max(1.0);

        // Downsample to draw ~1000 points to ensure good performance
        let step = (num_samples / 1000).max(1);

        let signal_color = Color::srgba(0.0, 0.8, 1.0, 0.45); // Cyan carrier wave
        let env_color = Color::srgb(1.0, 0.6, 0.0); // Orange envelope wave

        let mut prev_sig_point: Option<Vec2> = None;
        let mut prev_env_point: Option<Vec2> = None;

        for idx in (0..num_samples).step_by(step) {
            let t = idx as f32 / (num_samples - 1) as f32;
            let x = bottom_left.x + t * plot_width;

            // Carrier Wave
            let sig_val = signal[idx];
            let y_sig = plot_center.y + sig_val * scale_y;
            let sig_point = Vec2::new(x, y_sig);

            if let Some(prev) = prev_sig_point {
                gizmos.line_2d(prev, sig_point, signal_color);
            }
            prev_sig_point = Some(sig_point);

            // Envelope Wave
            let env_val = envelope[idx];
            let y_env = plot_center.y + env_val * scale_y;
            let env_point = Vec2::new(x, y_env);

            if let Some(prev) = prev_env_point {
                gizmos.line_2d(prev, env_point, env_color);
            }
            prev_env_point = Some(env_point);
        }

        // Draw Plot Titles and Legends
        draw_string(
            &mut gizmos,
            "Ultrasonic Echo Signal (Superposition)",
            Vec2::new(bottom_left.x, top_right.y + 10.0),
            8.0,
            Color::BLACK,
        );
        draw_string(
            &mut gizmos,
            "Distance (mm)",
            Vec2::new(plot_center.x - 50.0, bottom_left.y - 35.0),
            8.0,
            border_color,
        );

        draw_string(
            &mut gizmos,
            "Carrier Wave",
            Vec2::new(top_right.x - 220.0, top_right.y + 10.0),
            6.5,
            signal_color,
        );
        draw_string(
            &mut gizmos,
            "Envelope",
            Vec2::new(top_right.x - 90.0, top_right.y + 10.0),
            6.5,
            env_color,
        );

        // Display gain adjustment instructions
        let gain_text = format!("Gain: {:.1}x (+/- to adjust)", sensor.gain);
        draw_string(
            &mut gizmos,
            &gain_text,
            Vec2::new(bottom_left.x, bottom_left.y - 35.0),
            7.0,
            Color::BLACK,
        );
    }
}

// System to dynamically adjust the sensor's amplification factor
pub fn adjust_sensor_gain(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut component::UltrasonicSensor>,
) {
    for mut sensor in query.iter_mut() {
        if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
            sensor.gain = (sensor.gain + 0.5).min(20.0);
        }
        if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
            sensor.gain = (sensor.gain - 0.5).max(0.5);
        }
    }
}
