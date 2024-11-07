use std::{
    sync::{Condvar, Mutex},
    thread,
    time::Duration,
};

use log::trace;

#[derive(Debug)]
pub struct Keyboard {
    pressed_key: Mutex<u8>,
    key_pressed_cv: Condvar,
}
impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed_key: Mutex::new(0x0),
            key_pressed_cv: Condvar::new(),
        }
    }

    pub fn set_key(&self, key: u8) {
        let mut pressed_key_lock = self.pressed_key.lock().unwrap_or_else(|p| p.into_inner());
        *pressed_key_lock = key;

        trace!("Set pressed key to {}", *pressed_key_lock);

        self.key_pressed_cv.notify_all();
    }

    pub fn release_key(&self) {
        let mut pressed_key_lock = self.pressed_key.lock().unwrap_or_else(|p| p.into_inner());
        *pressed_key_lock = 0x0;

        trace!("Released button");
    }

    pub fn is_key_pressed(&self, key: u8) -> bool {
        trace!("Check if key is pressed");

        let pressed_key_lock = self.pressed_key.lock().unwrap_or_else(|p| p.into_inner());
        *pressed_key_lock == key
    }

    /// Blocks the thread until the key is pressed.
    pub fn wait_for_key(&self) -> u8 {
        trace!("Waiting for key press");
        let mut pressed_key_lock = self.pressed_key.lock().unwrap_or_else(|p| p.into_inner());

        while *pressed_key_lock == 0x0 {
            pressed_key_lock = self
                .key_pressed_cv
                .wait(pressed_key_lock)
                .unwrap_or_else(|p| p.into_inner());
        }

        trace!("Received key");
        *pressed_key_lock
    }
}
