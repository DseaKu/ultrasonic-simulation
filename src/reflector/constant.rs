pub mod defaults {
    pub const SPEED: f32 = 0.50;
    pub const ACCELERATION_FACTOR: f32 = 40.0;
}

pub mod bounds {
    pub const MIN_X: f32 = -250.0;
    pub const MAX_X: f32 = 5000.0;
    pub const MIN_Y: f32 = -220.0;
    pub const MAX_Y: f32 = 220.0;
}

pub mod spawn_position {
    pub const X: f32 = 2000.0;
    pub const Y: f32 = 0.0;
    pub const Z: f32 = 0.0;
}

pub mod dimensions {
    pub const WIDTH: f32 = 120.0;
    pub const HEIGHT: f32 = 120.0;
}
