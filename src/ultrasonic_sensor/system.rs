use super::bundle;
use super::component;
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn setup_sensor(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(bundle::UltrasonicSensorBundle::new(asset_server));
}

pub fn collect_sensor_data(
    spatial_query: SpatialQuery,
    time: Res<Time<Virtual>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &Transform,
        &mut component::UltrasonicSensor,
        &mut component::SensorHits,
        &mut component::HitHistory,
    )>,
    mut gizmos: Gizmos,
) {
    if time.is_paused() {
        for (transform, sensor, _, _) in query.iter() {
            let origin = transform.translation.xy();
            let forward_vec3 = transform.rotation * Vec3::X;
            let forward = Vec2::new(forward_vec3.x, forward_vec3.y).normalize_or_zero();

            let ray_count = sensor.ray_count;
            let beam_angle = sensor.beam_angle;

            let hit_color = Color::srgb(0.0, 1.0, 1.0); // Cyan
            let miss_color = Color::srgb(0.5, 0.5, 0.5); // Gray

            for i in 0..ray_count {
                let angle_offset = if ray_count > 1 {
                    let t = i as f32 / (ray_count - 1) as f32;
                    (t - 0.5) * beam_angle
                } else {
                    0.0
                };

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
                    gizmos.line_2d(origin, hit_point, hit_color);
                } else {
                    gizmos.line_2d(
                        origin,
                        origin + ray_direction * sensor.max_range,
                        miss_color,
                    );
                }
            }
        }
        return;
    }

    let mut target_velocity = 0.0;
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        target_velocity = -crate::reflector::constant::SPEED;
    } else if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        target_velocity = crate::reflector::constant::SPEED;
    }

    for (transform, mut sensor, mut sensor_hits, mut hit_history) in query.iter_mut() {
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
                let v = target_velocity;

                // 2. Time of Flight (t_delay)
                let delay = (2.0 * d_current) / c;

                // 3. Two-Way Doppler Shift (f_r) with exaggeration to make it visible.
                // Uses a linear approximation to avoid division singularities (sonic boom)
                // when exaggerated velocity approaches or exceeds the speed of sound.
                let factor = (1.0 - (2.0 * v * sensor.doppler_exaggeration) / c).clamp(
                    super::constant::MIN_DOPPLER_FACTOR,
                    super::constant::MAX_DOPPLER_FACTOR,
                );
                let doppler_freq = f_t * factor;

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

        // Calculate instantaneous Doppler frequency of hits in this frame
        let mut avg_doppler = f_t;
        if !sensor_hits.hits.is_empty() {
            let sum: f32 = sensor_hits.hits.iter().map(|h| h.doppler_freq).sum();
            avg_doppler = sum / sensor_hits.hits.len() as f32;
        }

        // Apply exponential moving average (EMA) to smooth out numerical noise
        let alpha = super::constant::signal::DOPPLER_SMOOTHING_ALPHA;
        sensor.smoothed_rx_frequency =
            sensor.smoothed_rx_frequency + alpha * (avg_doppler - sensor.smoothed_rx_frequency);
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
        let fs = super::constant::signal::SAMPLE_RATE; // 200 kHz sample rate
        let dt_s = 1.0 / fs;

        let min_dist = super::constant::signal::MIN_DISTANCE; // Start at negative distance to show full transmitted pulse
        let max_dist = sensor.max_range;

        let t_start = 2.0 * min_dist / sensor.speed_of_sound;
        let t_end = 2.0 * max_dist / sensor.speed_of_sound;
        let t_span = t_end - t_start;
        let num_samples = (t_span / dt_s).ceil() as usize;

        let mut signal = vec![0.0; num_samples];
        let mut time_axis = vec![0.0; num_samples];
        for (j, time) in time_axis.iter_mut().enumerate() {
            *time = t_start + j as f32 * dt_s;
        }

        let sigma = sensor.pulse_width;
        let sigma_sq = sigma * sigma;

        // Synthesize the transmitted pulse ("main bang") centered at t = 0 (amplitude normalized to 1.0)
        let tx_amplitude = super::constant::signal::TX_AMPLITUDE * sensor.gain;
        let tx_t_start = -super::constant::signal::SIGMA_MULTIPLIER * sigma;
        let tx_t_end = super::constant::signal::SIGMA_MULTIPLIER * sigma;
        let tx_idx_start = ((tx_t_start - t_start) / dt_s) as usize;
        let tx_idx_end = (((tx_t_end - t_start) / dt_s) as usize).min(num_samples - 1);

        for j in tx_idx_start..=tx_idx_end {
            let t = time_axis[j];
            let diff = t; // t - t_tx where t_tx = 0
            let env = (-diff * diff / (2.0 * sigma_sq)).exp();
            let wave = env * (2.0 * std::f32::consts::PI * sensor.frequency * diff).cos();
            signal[j] += wave * tx_amplitude;
        }

        for hit in sensor_hits.hits.iter() {
            let t_d = hit.delay;
            let f_r = hit.doppler_freq;
            let dist = hit.distance;

            // Physical distance attenuation: inverse square law scaled by gain and normalized by ray count.
            // Because each ray represents a fraction of the wavefront energy, the sum of the ray echoes
            // is normalized by the ray count so it cannot exceed the transmitted pulse amplitude.
            let atten = (super::constant::signal::ATTENUATION_REF_DIST
                / dist.max(super::constant::signal::ATTENUATION_REF_DIST))
            .powi(2)
                * sensor.gain
                / (sensor.ray_count as f32);

            // Sparse evaluation: within +/- 4 sigma
            let echo_t_start = t_d - super::constant::signal::SIGMA_MULTIPLIER * sigma;
            let echo_t_end = t_d + super::constant::signal::SIGMA_MULTIPLIER * sigma;

            let idx_start = ((echo_t_start - t_start) / dt_s) as usize;
            let idx_end = (((echo_t_end - t_start) / dt_s) as usize).min(num_samples - 1);

            for j in idx_start..=idx_end {
                let t = time_axis[j];
                let diff = t - t_d;
                let env = (-diff * diff / (2.0 * sigma_sq)).exp();
                let wave = env * (2.0 * std::f32::consts::PI * f_r * diff).cos();
                signal[j] += wave * atten;
            }
        }

        // 1. Zero-phase low-pass filter envelope detection
        let mut envelope = vec![0.0; num_samples];
        let alpha = super::constant::signal::ENVELOPE_LOWPASS_ALPHA; // Cutoff frequency (~2.5 kHz) to smooth out the 40 kHz carrier ripples

        // Forward filter pass
        let mut filter_state = 0.0;
        for j in 0..num_samples {
            let rect_val = signal[j].abs();
            filter_state = alpha * rect_val + (1.0 - alpha) * filter_state;
            envelope[j] = filter_state;
        }

        // Backward filter pass for zero-phase alignment and double-stage smoothing
        let mut smooth_envelope = vec![0.0; num_samples];
        let mut filter_state_back = 0.0;
        for j in (0..num_samples).rev() {
            let val = envelope[j];
            filter_state_back = alpha * val + (1.0 - alpha) * filter_state_back;
            smooth_envelope[j] = filter_state_back * std::f32::consts::FRAC_PI_2; // Multiply by pi/2 to restore peak amplitude
        }

        ultrasonic_signal.time_axis = time_axis;
        ultrasonic_signal.signal = signal;
        ultrasonic_signal.envelope = smooth_envelope;
    }
}

pub fn setup_time_scale(mut time: ResMut<Time<Virtual>>) {
    time.set_relative_speed(super::constant::DEFAULT_TIME_SCALE);
}

pub fn adjust_time_scale(keyboard: Res<ButtonInput<KeyCode>>, mut time: ResMut<Time<Virtual>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }

    let mut speed = time.relative_speed();
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        speed = (speed - super::constant::TIME_SCALE_STEP).max(super::constant::MIN_TIME_SCALE);
        time.set_relative_speed(speed);
    }
    if keyboard.just_pressed(KeyCode::BracketRight) {
        speed = (speed + super::constant::TIME_SCALE_STEP).min(super::constant::MAX_TIME_SCALE);
        time.set_relative_speed(speed);
    }
}

// Plot system using Bevy Gizmos to visualize the synthetic signals
pub fn plot_sensor_signal(
    query: Query<(&component::UltrasonicSensor, &component::UltrasonicSignal)>,
    mut gizmos: Gizmos,
    time: Res<Time<Virtual>>,
) {
    for (sensor, ultrasonic_signal) in query.iter() {
        if ultrasonic_signal.signal.is_empty() {
            continue;
        }

        // Define plotting region
        let plot_width = super::constant::plot::WIDTH;
        let plot_height = super::constant::plot::HEIGHT;
        let plot_center = Vec2::new(
            super::constant::plot::CENTER_X,
            super::constant::plot::CENTER_Y,
        );

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

        // Distances alignment constants
        let min_dist = super::constant::signal::MIN_DISTANCE;
        let max_range = sensor.max_range;
        let total_dist = max_range - min_dist;

        let get_x = |d: f32| -> f32 { bottom_left.x + ((d - min_dist) / total_dist) * plot_width };

        // Draw negative start tick
        {
            let x = get_x(min_dist);
            gizmos.line_2d(
                Vec2::new(x, bottom_left.y),
                Vec2::new(x, bottom_left.y - super::constant::plot::TICK_LENGTH),
                border_color,
            );
            let label = format!("{}", min_dist as i32);
            gizmos.text_2d(
                Vec2::new(x, bottom_left.y - super::constant::plot::TICK_LABEL_OFFSET),
                &label,
                super::constant::plot::TICK_LABEL_SIZE,
                Vec2::ZERO,
                border_color,
            );
        }

        // Draw vertical grid ticks and labels (every 100 distance units starting at 0)
        let num_ticks = super::constant::plot::NUM_TICKS;
        for i in 0..=num_ticks {
            let dist = (i * super::constant::plot::TICK_SPACING) as f32;
            let x = get_x(dist);

            // Grid line
            gizmos.line_2d(
                Vec2::new(x, bottom_left.y),
                Vec2::new(x, top_right.y),
                grid_color,
            );

            // Tick mark
            gizmos.line_2d(
                Vec2::new(x, bottom_left.y),
                Vec2::new(x, bottom_left.y - super::constant::plot::TICK_LENGTH),
                border_color,
            );

            // Tick label (distance in mm/units)
            let label = format!("{}", dist as i32);
            gizmos.text_2d(
                Vec2::new(x, bottom_left.y - super::constant::plot::TICK_LABEL_OFFSET),
                &label,
                super::constant::plot::TICK_LABEL_SIZE,
                Vec2::ZERO,
                border_color,
            );
        }

        // Plot signal and envelope
        let signal = &ultrasonic_signal.signal;
        let envelope = &ultrasonic_signal.envelope;
        let time_axis = &ultrasonic_signal.time_axis;
        let num_samples = signal.len();

        // Since the maximum possible amplitude of the signal is normalized to 1.0 (at gain = 1.0),
        // we use a fixed scaling factor of 1.0.
        let max_possible_amp = 1.0;
        let scale_y = half_h / max_possible_amp;

        // Downsample to draw ~1000 points to ensure good performance
        let step = (num_samples / super::constant::plot::DOWNSAMPLE_TARGET).max(1);

        let signal_color = Color::srgba(0.0, 0.8, 1.0, 0.45); // Cyan carrier wave
        let env_color = Color::srgb(1.0, 0.6, 0.0); // Orange envelope wave

        let mut prev_sig_point: Option<Vec2> = None;
        let mut prev_env_point: Option<Vec2> = None;

        for idx in (0..num_samples).step_by(step) {
            let t = time_axis[idx];
            let dist = t * sensor.speed_of_sound / 2.0;
            let x = get_x(dist);

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

        // Draw Plot Titles and Legends (using correct text_2d alignment bounds)
        gizmos.text_2d(
            Vec2::new(
                bottom_left.x,
                top_right.y + super::constant::plot::MARGIN_Y,
            ),
            "Ultrasonic Echo Signal (Superposition)",
            super::constant::plot::TITLE_SIZE,
            Vec2::new(-0.5, 0.0), // Left aligned
            Color::BLACK,
        );
        gizmos.text_2d(
            Vec2::new(
                plot_center.x,
                bottom_left.y - super::constant::plot::LABEL_OFFSET_Y,
            ),
            "Distance (mm)",
            super::constant::plot::AXIS_LABEL_SIZE,
            Vec2::ZERO, // Centered
            border_color,
        );

        // Draw live Tx/Rx frequency readouts to demonstrate Doppler shift numerically
        let freq_text = format!(
            "Tx Freq: {:.1} kHz | Rx Freq: {:.1} kHz",
            sensor.frequency / 1000.0,
            sensor.smoothed_rx_frequency / 1000.0
        );
        gizmos.text_2d(
            Vec2::new(
                plot_center.x + super::constant::plot::FREQ_LABEL_OFFSET_X,
                top_right.y + super::constant::plot::MARGIN_Y,
            ),
            &freq_text,
            super::constant::plot::LEGEND_SIZE,
            Vec2::ZERO, // Centered
            Color::BLACK,
        );

        gizmos.text_2d(
            Vec2::new(
                top_right.x - super::constant::plot::LEGEND_SPACING,
                top_right.y + super::constant::plot::MARGIN_Y,
            ),
            "Carrier Wave",
            super::constant::plot::LEGEND_SIZE,
            Vec2::new(0.5, 0.0), // Right aligned relative to position
            signal_color,
        );
        gizmos.text_2d(
            Vec2::new(top_right.x, top_right.y + super::constant::plot::MARGIN_Y),
            "Envelope",
            super::constant::plot::LEGEND_SIZE,
            Vec2::new(0.5, 0.0), // Right aligned relative to position
            env_color,
        );

        // Display gain, Doppler exaggeration, and time scale instructions
        let time_scale_text = if time.is_paused() {
            "Paused (Space)".to_string()
        } else {
            format!("{:.2}x ([/]/Space)", time.relative_speed())
        };
        let gain_text = format!(
            "Gain: {:.1}x (+/-) | Doppler: {:.0}x (</>) | Time Scale: {}",
            sensor.gain, sensor.doppler_exaggeration, time_scale_text
        );
        gizmos.text_2d(
            Vec2::new(
                bottom_left.x,
                bottom_left.y - super::constant::plot::LABEL_OFFSET_Y,
            ),
            &gain_text,
            super::constant::plot::INSTRUCTION_SIZE,
            Vec2::new(-0.5, 0.0), // Left aligned
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
            sensor.gain = (sensor.gain + super::constant::GAIN_STEP).min(super::constant::MAX_GAIN);
        }
        if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
            sensor.gain = (sensor.gain - super::constant::GAIN_STEP).max(super::constant::MIN_GAIN);
        }
    }
}

// System to dynamically adjust the Doppler shift exaggeration factor
pub fn adjust_doppler_exaggeration(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut component::UltrasonicSensor>,
) {
    for mut sensor in query.iter_mut() {
        if keyboard.just_pressed(KeyCode::Comma) {
            sensor.doppler_exaggeration = (sensor.doppler_exaggeration
                - super::constant::DOPPLER_EXAGGERATION_STEP)
                .max(super::constant::MIN_DOPPLER_EXAGGERATION);
        }
        if keyboard.just_pressed(KeyCode::Period) {
            sensor.doppler_exaggeration = (sensor.doppler_exaggeration
                + super::constant::DOPPLER_EXAGGERATION_STEP)
                .min(super::constant::MAX_DOPPLER_EXAGGERATION);
        }
    }
}
