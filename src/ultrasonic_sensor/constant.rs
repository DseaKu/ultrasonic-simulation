pub const FREQUENCY_TRANSMITTED: f32 = 40_000.0;
pub const SPEED_OF_SOUND: f32 = 343_000.0;

pub const MIN_DOPPLER_FACTOR: f32 = 0.25;
pub const MAX_DOPPLER_FACTOR: f32 = 2.25;

pub const DOPPLER_EXAGGERATION_STEP: f32 = 50.0;
pub const MIN_DOPPLER_EXAGGERATION: f32 = 0.0;
pub const MAX_DOPPLER_EXAGGERATION: f32 = 2000.0;

pub const TIME_SCALE_STEP: f32 = 0.05;
pub const MIN_TIME_SCALE: f32 = 0.05;
pub const MAX_TIME_SCALE: f32 = 2.0;
pub const DEFAULT_TIME_SCALE: f32 = 0.2;

pub const GAIN_STEP: f32 = 0.5;
pub const MIN_GAIN: f32 = 0.5;
pub const MAX_GAIN: f32 = 20.0;

pub mod signal {
    pub const SAMPLE_RATE: f32 = 200_000.0;
    pub const MIN_DISTANCE: f32 = -100.0;
    pub const TX_AMPLITUDE: f32 = 1.0;
    pub const SIGMA_MULTIPLIER: f32 = 4.0;
    pub const ATTENUATION_REF_DIST: f32 = 150.0;
    pub const ENVELOPE_LOWPASS_ALPHA: f32 = 0.08;
    pub const DOPPLER_SMOOTHING_ALPHA: f32 = 0.15;
}

pub mod plot {
    pub const WIDTH: f32 = 1000.0;
    pub const HEIGHT: f32 = 160.0;
    pub const CENTER_X: f32 = 0.0;
    pub const CENTER_Y: f32 = -250.0;
    pub const TICK_LENGTH: f32 = 5.0;
    pub const TICK_LABEL_OFFSET: f32 = 15.0;
    pub const TICK_LABEL_SIZE: f32 = 11.0;
    pub const NUM_TICKS: usize = 8;
    pub const TICK_SPACING: usize = 100;
    pub const DOWNSAMPLE_TARGET: usize = 1000;

    pub const TITLE_SIZE: f32 = 14.0;
    pub const AXIS_LABEL_SIZE: f32 = 13.0;
    pub const LEGEND_SIZE: f32 = 12.0;
    pub const INSTRUCTION_SIZE: f32 = 12.0;

    pub const MARGIN_Y: f32 = 10.0;
    pub const LABEL_OFFSET_Y: f32 = 35.0;
    pub const FREQ_LABEL_OFFSET_X: f32 = 130.0;
    pub const LEGEND_SPACING: f32 = 100.0;
}

pub mod spawn_position {
    pub const X: f32 = -400.0;
    pub const Y: f32 = 0.0;
    pub const Z: f32 = 0.0;
}
