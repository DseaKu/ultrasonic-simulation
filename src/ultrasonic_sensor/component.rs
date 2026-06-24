use super::constant;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct UltrasonicSensor {
    pub frequency: f32,
    pub speed_of_sound: f32,
    pub pulse_width: f32,
    pub beam_angle: f32,
    pub ray_count: usize,
    pub max_range: f32,
    pub gain_db: f32,
    pub smoothed_rx_frequency: f32,
    pub doppler_exaggeration: f32,
    pub show_rays: bool,
    pub show_carrier_wave: bool,
    pub show_rx_frequency: bool,
    pub temperature: f32,
    pub tx_amplitude: f32,
}

impl UltrasonicSensor {
    pub fn new() -> Self {
        Self {
            frequency: constant::FREQUENCY_TRANSMITTED,
            speed_of_sound: constant::SPEED_OF_SOUND,
            pulse_width: constant::defaults::PULSE_WIDTH,
            beam_angle: constant::defaults::BEAM_ANGLE_DEG.to_radians(),
            ray_count: constant::defaults::RAY_COUNT,
            max_range: constant::defaults::MAX_RANGE,
            gain_db: constant::defaults::GAIN_DB,
            smoothed_rx_frequency: constant::FREQUENCY_TRANSMITTED,
            doppler_exaggeration: constant::defaults::DOPPLER_EXAGGERATION,
            show_rays: constant::defaults::SHOW_RAYS,
            show_carrier_wave: constant::defaults::SHOW_CARRIER_WAVE,
            show_rx_frequency: constant::defaults::SHOW_RX_FREQUENCY,
            temperature: constant::defaults::TEMPERATURE,
            tx_amplitude: constant::defaults::TX_AMPLITUDE,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RayHit {
    pub entity: Entity,
    pub point: Vec2,
    pub distance: f32,
    pub velocity: f32,
    pub delay: f32,
    pub doppler_freq: f32,
}

#[derive(Component, Default, Debug, Clone)]
pub struct SensorHits {
    pub hits: Vec<RayHit>,
}

impl SensorHits {
    pub fn new() -> Self {
        Self { hits: Vec::new() }
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct HitHistory {
    pub distances: HashMap<(Entity, usize), f32>,
}

impl HitHistory {
    pub fn new() -> Self {
        Self {
            distances: HashMap::new(),
        }
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct UltrasonicSignal {
    pub time_axis: Vec<f32>,
    pub signal: Vec<f32>,
    pub envelope: Vec<f32>,
}

impl UltrasonicSignal {
    pub fn new() -> Self {
        Self {
            time_axis: Vec::new(),
            signal: Vec::new(),
            envelope: Vec::new(),
        }
    }
}
