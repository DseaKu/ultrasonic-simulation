use bevy::prelude::*;
use bevy::input::keyboard::{Key, KeyboardInput};

pub fn test_system(mut evr: EventReader<KeyboardInput>) {
    for ev in evr.read() {
        if let Key::Character(ref c) = ev.logical_key {
            let s: &str = c.as_str();
        }
        if ev.key_code == KeyCode::Enter {
            
        }
        if ev.state.is_pressed() {
            
        }
    }
}
