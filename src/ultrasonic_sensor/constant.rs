pub const FREQUENCY_TRANSMITTED: f32 = 40_000.0;
pub const SPEED_OF_SOUND: f32 = 343_000.0;

pub const MIN_DOPPLER_FACTOR: f32 = 0.25;
pub const MAX_DOPPLER_FACTOR: f32 = 2.25;

pub const MIN_DOPPLER_EXAGGERATION: f32 = 0.0;
pub const MAX_DOPPLER_EXAGGERATION: f32 = 2000.0;

pub mod defaults {
    pub const PULSE_WIDTH: f32 = 0.0005;
    pub const BEAM_ANGLE_DEG: f32 = 20.0;
    pub const RAY_COUNT: usize = 128;
    pub const MAX_RANGE: f32 = 5000.0;
    pub const GAIN_DB: f32 = 40.0;
    pub const DOPPLER_EXAGGERATION: f32 = 1.0;
    pub const SHOW_RAYS: bool = false;
    pub const SHOW_CARRIER_WAVE: bool = false;
    pub const SHOW_RX_FREQUENCY: bool = false;
    pub const TEMPERATURE: f32 = 20.0;
    pub const TX_AMPLITUDE: f32 = 1.0;
}

pub mod signal {
    pub const SAMPLE_RATE: f32 = 200_000.0;
    pub const MIN_DISTANCE: f32 = -100.0;
    pub const TX_AMPLITUDE: f32 = 5.0;
    pub const SIGMA_MULTIPLIER: f32 = 4.0;
    pub const ATTENUATION_REF_DIST: f32 = 150.0;
    pub const DOPPLER_SMOOTHING_ALPHA: f32 = 0.15;
}

pub mod plot {
    pub const WIDTH: f32 = 1000.0;
    pub const HEIGHT: f32 = 160.0;
    pub const CENTER_X: f32 = 0.0;
    pub const CENTER_Y: f32 = -5000.0;
    pub const TICK_LENGTH: f32 = 5.0;
    pub const TICK_LABEL_OFFSET: f32 = 15.0;
    pub const TICK_LABEL_SIZE: f32 = 11.0;
    pub const NUM_TICKS: usize = 10;
    pub const TICK_SPACING: usize = 500;
    pub const DOWNSAMPLE_TARGET: usize = 20000; // High enough to never downsample, avoiding 40kHz aliasing

    pub const AXIS_LABEL_SIZE: f32 = 13.0;
    pub const LEGEND_SIZE: f32 = 12.0;
    pub const INSTRUCTION_SIZE: f32 = 12.0;

    pub const LEGEND_SPACING: f32 = 100.0;
}

pub mod spawn_position {
    pub const X: f32 = -400.0;
    pub const Y: f32 = 0.0;
    pub const Z: f32 = 0.0;
}
