use std::sync::{Condvar, Mutex};

use bevy::prelude::*;

#[derive(Debug)]
pub struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Keyboard::new())
            .add_systems(FixedPreUpdate, update_keyboard_keys_system);
    }
}

fn update_keyboard_keys_system(keys: Res<ButtonInput<KeyCode>>, keyboard: Res<Keyboard>) {
    for keycode in keys.get_just_pressed() {
        let key: u8 = match get_key_from_keycode(keycode) {
            Some(key) => key,
            None => continue,
        };

        keyboard.set_key(Some(key));
        debug!("Pressed key: {}", key);
        return;
    }

    keyboard.set_key(None);
}

fn get_key_from_keycode(keycode: &KeyCode) -> Option<u8> {
    let key = match keycode {
        KeyCode::Digit1 => 0x1,
        KeyCode::Digit2 => 0x2,
        KeyCode::Digit3 => 0x3,
        KeyCode::Digit4 => 0xC,
        KeyCode::KeyQ => 0x4,
        KeyCode::KeyW => 0x5,
        KeyCode::KeyE => 0x6,
        KeyCode::KeyR => 0xD,
        KeyCode::KeyA => 0x7,
        KeyCode::KeyS => 0x8,
        KeyCode::KeyD => 0x9,
        KeyCode::KeyF => 0xE,
        KeyCode::KeyZ => 0xA,
        KeyCode::KeyX => 0x0,
        KeyCode::KeyC => 0xB,
        KeyCode::KeyV => 0xF,
        _ => return None,
    };

    Some(key)
}

#[derive(Debug, Resource)]
pub struct Keyboard {
    pressed_key: Mutex<Option<u8>>,
    key_pressed_cv: Condvar,
}
impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed_key: Mutex::new(None),
            key_pressed_cv: Condvar::new(),
        }
    }

    pub fn set_key(&self, key: Option<u8>) {
        let mut pressed_key_lock = self.pressed_key.lock().unwrap_or_else(|p| p.into_inner());
        *pressed_key_lock = key;

        trace!("Set pressed key to {:?}", pressed_key_lock);

        self.key_pressed_cv.notify_all();
    }

    pub fn is_key_pressed(&self, key: u8) -> bool {
        trace!("Check if key is pressed");

        let pressed_key_lock = self.pressed_key.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(pressed_key) = *pressed_key_lock {
            pressed_key == key
        } else {
            false
        }
    }

    /// Blocks the thread until the key is pressed.
    pub fn wait_for_any_key_press(&self) -> u8 {
        trace!("Waiting for key press");
        let mut pressed_key_lock = self.pressed_key.lock().unwrap_or_else(|p| p.into_inner());

        while pressed_key_lock.is_none() {
            pressed_key_lock = self
                .key_pressed_cv
                .wait(pressed_key_lock)
                .unwrap_or_else(|p| p.into_inner());
        }

        trace!("Received key");
        pressed_key_lock.expect("Failed to return awaited key press")
    }
}
