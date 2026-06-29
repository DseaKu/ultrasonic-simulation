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
    mut query: Query<(
        &Transform,
        &mut component::UltrasonicSensor,
        &mut component::SensorHits,
        &mut component::HitHistory,
    )>,
    reflector_query: Query<&crate::reflector::component::Reflector>,
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
                    if sensor.show_rays {
                        gizmos.line_2d(origin, hit_point, hit_color);
                    }
                } else {
                    if sensor.show_rays {
                        gizmos.line_2d(
                            origin,
                            origin + ray_direction * sensor.max_range,
                            miss_color,
                        );
                    }
                }
            }
            if sensor.show_rx_frequency {
                gizmos.text_2d(
                    Vec2::new(0.0, 330.0),
                    &format!("Transmitted Frequency: {:.2} kHz", sensor.frequency / 1000.0),
                    24.0,
                    Vec2::new(0.0, 0.0),
                    Color::BLACK,
                );
                gizmos.text_2d(
                    Vec2::new(0.0, 300.0), // Fixed position high above the simulation area
                    &format!("Received Frequency: {:.2} kHz", sensor.smoothed_rx_frequency / 1000.0),
                    24.0,
                    Vec2::new(0.0, 0.0),
                    Color::BLACK,
                );
            }
        }
        return;
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
                let mut v = 0.0;
                if let Ok(hit_reflector) = reflector_query.get(hit.entity) {
                    v = hit_reflector.current_velocity.x;
                }

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
                if sensor.show_rays {
                    gizmos.line_2d(origin, hit_point, hit_color);
                }
            } else {
                // Draw a dim gray line representing the ray's path in the air
                if sensor.show_rays {
                    gizmos.line_2d(
                        origin,
                        origin + ray_direction * sensor.max_range,
                        miss_color,
                    );
                }
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

        if sensor.show_rx_frequency {
            gizmos.text_2d(
                Vec2::new(0.0, 330.0),
                &format!("Transmitted Frequency: {:.2} kHz", sensor.frequency / 1000.0),
                24.0,
                Vec2::new(0.0, 0.0),
                Color::BLACK,
            );
            gizmos.text_2d(
                Vec2::new(0.0, 300.0), // Fixed position high above the simulation area
                &format!("Received Frequency: {:.2} kHz", sensor.smoothed_rx_frequency / 1000.0),
                24.0,
                Vec2::new(0.0, 0.0),
                Color::BLACK,
            );
        }
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

        let assumed_c = super::constant::SPEED_OF_SOUND;
        let t_start = 2.0 * min_dist / assumed_c;
        let t_end = 2.0 * max_dist / assumed_c;
        let t_span = t_end - t_start;
        let num_samples = (t_span / dt_s).ceil() as usize;


        let mut time_axis = vec![0.0; num_samples];
        for (j, time) in time_axis.iter_mut().enumerate() {
            *time = t_start + j as f32 * dt_s;
        }

        let sigma = sensor.pulse_width;
        let sigma_sq = sigma * sigma;
        let mut i_signal = vec![0.0; num_samples];
        let mut q_signal = vec![0.0; num_samples];
        let f_tx = sensor.frequency;

        // Synthesize the transmitted pulse ("main bang") centered at t = 0
        let tx_amplitude = super::constant::signal::TX_AMPLITUDE * sensor.tx_amplitude;
        let tx_t_start = -super::constant::signal::SIGMA_MULTIPLIER * sigma;
        let tx_t_end = super::constant::signal::SIGMA_MULTIPLIER * sigma;
        let tx_idx_start = ((tx_t_start - t_start) / dt_s) as usize;
        let tx_idx_end = (((tx_t_end - t_start) / dt_s) as usize).min(num_samples - 1);

        for j in tx_idx_start..=tx_idx_end {
            let t = time_axis[j];
            let env = (-t * t / (2.0 * sigma_sq)).exp();
            let phase = 2.0 * std::f32::consts::PI * f_tx * t;
            i_signal[j] += env * phase.cos() * tx_amplitude;
            q_signal[j] += env * phase.sin() * tx_amplitude;
        }

        for hit in sensor_hits.hits.iter() {
            let t_d = hit.delay;
            let f_r = hit.doppler_freq;
            let dist = hit.distance;

            // Convert dB to linear voltage gain multiplier
            let linear_gain = 10.0f32.powf(sensor.gain_db / 20.0);

            // Physical distance attenuation: inverse square law scaled by gain and normalized by ray count.
            // Because each ray represents a fraction of the wavefront energy, the sum of the ray echoes
            // is normalized by the ray count so it cannot exceed the transmitted pulse amplitude.
            let atten = tx_amplitude 
                * (super::constant::signal::ATTENUATION_REF_DIST
                / dist.max(super::constant::signal::ATTENUATION_REF_DIST))
            .powi(2)
                * linear_gain
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
                let phase = 2.0 * std::f32::consts::PI * f_r * diff;
                i_signal[j] += env * phase.cos() * atten;
                q_signal[j] += env * phase.sin() * atten;
            }
        }

        // Mathematically perfect IQ Envelope Detection
        // The true envelope is the magnitude of the complex analytic signal
        let mut true_envelope = vec![0.0; num_samples];
        for j in 0..num_samples {
            true_envelope[j] = (i_signal[j] * i_signal[j] + q_signal[j] * q_signal[j]).sqrt();
        }

        ultrasonic_signal.time_axis = time_axis;
        ultrasonic_signal.signal = i_signal;
        ultrasonic_signal.envelope = true_envelope;
    }
}

pub fn toggle_pause(keyboard: Res<ButtonInput<KeyCode>>, mut time: ResMut<Time<Virtual>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
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

        // Shift the 0-amplitude axis down slightly to give more room for positive envelope peaks
        let zero_y = plot_center.y - half_h * 0.4;

        // Center line (zero amplitude)
        gizmos.line_2d(
            Vec2::new(bottom_left.x, zero_y),
            Vec2::new(top_right.x, zero_y),
            grid_color,
        );

        // Distances alignment constants
        let min_dist = super::constant::signal::MIN_DISTANCE;
        let max_range = sensor.max_range;
        let total_dist = max_range - min_dist;

        let get_x = |d: f32| -> f32 { bottom_left.x + ((d - min_dist) / total_dist) * plot_width };


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

            // Tick label (distance in meters, unitless numbers)
            let label = format!("{:.1}", dist / 1000.0);
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

        let assumed_c = super::constant::SPEED_OF_SOUND;
        for idx in (0..num_samples).step_by(step) {
            let t = time_axis[idx];
            // The plot maps time back to distance using the sensor's assumed speed of sound
            let dist = t * assumed_c / 2.0;
            let x = get_x(dist);

            // Carrier Wave
            if sensor.show_carrier_wave {
                // Simulate ADC/Op-Amp voltage clipping relative to new zero axis
                let sig_val = signal[idx].clamp(-0.6, 1.4);
                let y_sig = zero_y + sig_val * scale_y;
                let sig_point = Vec2::new(x, y_sig);

                if let Some(prev) = prev_sig_point {
                    gizmos.line_2d(prev, sig_point, signal_color);
                }
                prev_sig_point = Some(sig_point);
            }

            // Envelope Wave
            // Clamped to match the shifted plot boundaries
            let env_val = envelope[idx].clamp(-0.6, 1.4);
            let y_env = zero_y + env_val * scale_y;
            let env_point = Vec2::new(x, y_env);

            if let Some(prev) = prev_env_point {
                gizmos.line_2d(prev, env_point, env_color);
            }
            prev_env_point = Some(env_point);
        }

        // Draw Plot Legends (using correct text_2d alignment bounds)
        if sensor.show_carrier_wave {
            gizmos.text_2d(
                Vec2::new(
                    top_right.x - super::constant::plot::LEGEND_SPACING,
                    top_right.y + 25.0,
                ),
                "Carrier Wave",
                super::constant::plot::LEGEND_SIZE,
                Vec2::new(0.5, 0.0), // Right aligned relative to position
                signal_color,
            );
        }
        gizmos.text_2d(
            Vec2::new(top_right.x, top_right.y + 25.0),
            "Envelope",
            super::constant::plot::LEGEND_SIZE,
            Vec2::new(0.5, 0.0), // Right aligned relative to position
            env_color,
        );

        // Y-axis Label
        gizmos.text_2d(
            Vec2::new(bottom_left.x, top_right.y + 25.0),
            "Amplitude (a.u.)",
            super::constant::plot::AXIS_LABEL_SIZE,
            Vec2::new(0.0, 0.0), // Left aligned, above the left edge
            border_color,
        );

        gizmos.text_2d(
            Vec2::new(
                plot_center.x,
                bottom_left.y - 35.0,
            ),
            "Distance (m) →",
            super::constant::plot::AXIS_LABEL_SIZE,
            Vec2::ZERO, // Centered
            border_color,
        );

        // Display paused state at top center
        if time.is_paused() {
            gizmos.text_2d(
                Vec2::new(plot_center.x, top_right.y + 25.0),
                "PAUSED",
                super::constant::plot::INSTRUCTION_SIZE,
                Vec2::new(0.5, 0.0), // Center aligned
                Color::srgb(1.0, 0.2, 0.2), // Red so it's obvious
            );
        }
    }
}

pub fn egui_settings_panel(
    mut contexts: bevy_egui::EguiContexts,
    mut sensor_query: Query<&mut component::UltrasonicSensor>,
    mut reflector_query: Query<&mut crate::reflector::component::Reflector>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    if let Ok(mut sensor) = sensor_query.single_mut()
        && let Ok(mut reflector) = reflector_query.single_mut()
    {
        bevy_egui::egui::Window::new("Settings")
            .anchor(bevy_egui::egui::Align2::RIGHT_TOP, bevy_egui::egui::vec2(-10.0, 10.0))
            .default_open(false)
            .show(ctx, |ui| {
                ui.add_space(20.0);
                ui.heading("Settings");
                ui.separator();
                
                ui.add_space(20.0);
                ui.label("Gain (dB)");
                ui.add(bevy_egui::egui::DragValue::new(&mut sensor.gain_db)
                    .range(0.0..=100.0)
                    .speed(0.5));

                ui.add_space(10.0);
                ui.label("TX Power Amplitude");
                ui.add(bevy_egui::egui::DragValue::new(&mut sensor.tx_amplitude)
                    .range(0.1..=10.0)
                    .speed(0.1));

                ui.add_space(10.0);
                ui.label("Pulse Width (ms)");
                let mut pulse_ms = sensor.pulse_width * 1000.0;
                if ui.add(bevy_egui::egui::DragValue::new(&mut pulse_ms)
                    .range(0.01..=10.0)
                    .speed(0.01)).changed() {
                    sensor.pulse_width = pulse_ms / 1000.0;
                }

                ui.add_space(10.0);
                ui.label("TX Frequency (kHz)");
                let mut freq_khz = sensor.frequency / 1000.0;
                if ui.add(bevy_egui::egui::DragValue::new(&mut freq_khz)
                    .range(0.1..=100.0)
                    .speed(0.5)).changed() {
                    sensor.frequency = freq_khz * 1000.0;
                }

                ui.add_space(10.0);
                ui.label("Doppler Exaggeration (x)");
                ui.add(bevy_egui::egui::DragValue::new(&mut sensor.doppler_exaggeration)
                    .range(super::constant::MIN_DOPPLER_EXAGGERATION..=super::constant::MAX_DOPPLER_EXAGGERATION)
                    .speed(10.0));

                ui.add_space(10.0);
                ui.label("Temperature (C)");
                if ui.add(bevy_egui::egui::DragValue::new(&mut sensor.temperature).speed(0.5)).changed() {
                    sensor.speed_of_sound = (331.3 + 0.606 * sensor.temperature) * 1000.0;
                }
                ui.add_space(10.0);
                ui.label("Reflector Speed (m/s)");
                ui.add(bevy_egui::egui::DragValue::new(&mut reflector.speed)
                    .range(0.1..=10.0)
                    .speed(0.1));

                ui.add_space(10.0);
                ui.label("Reflector Width (mm)");
                ui.add(bevy_egui::egui::DragValue::new(&mut reflector.width)
                    .range(10.0..=1000.0)
                    .speed(1.0));

                ui.add_space(10.0);
                ui.label("Reflector Height (mm)");
                ui.add(bevy_egui::egui::DragValue::new(&mut reflector.height)
                    .range(10.0..=1000.0)
                    .speed(1.0));

                ui.add_space(10.0);
                ui.checkbox(&mut sensor.show_rays, "Show Ultrasonic Rays");
                ui.checkbox(&mut sensor.show_carrier_wave, "Show Carrier Wave");
                ui.checkbox(&mut sensor.show_rx_frequency, "Show Rx Frequency at Reflector");
            });
    }
}
