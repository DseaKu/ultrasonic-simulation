use super::constants;
use bevy::prelude::*;

#[derive(Component)]
pub struct UltrasonicSenosr {
    pub frequency: f32,
    pub speed_of_sound: f32,
    pub pulse_width: f32,
    pub beam_angle: f32,
    pub ray_count: usize,
}

impl UltrasonicSenosr {
    pub fn new() -> Self {
        Self {
            frequency: constants::FREQUENCY_TRANSMITTED,
            speed_of_sound: constants::SPEED_OF_SOUND,
            pulse_width: 1.0,
            beam_angle: 1.0,
            ray_count: 1,
        }
    }
}
