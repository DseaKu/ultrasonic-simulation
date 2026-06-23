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
}

impl UltrasonicSensor {
    pub fn new() -> Self {
        Self {
            frequency: constant::FREQUENCY_TRANSMITTED,
            speed_of_sound: constant::SPEED_OF_SOUND,
            pulse_width: 0.001, // 1 ms pulse width by default
            beam_angle: 30.0f32.to_radians(), // 30 degrees beam spread
            ray_count: 32, // 32 rays in the cone
            max_range: 800.0, // Max distance the rays can travel
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



