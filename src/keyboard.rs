use std::{thread, time::Duration};

#[derive(Debug)]
pub struct Keyboard {
    pressed_key: u8,
}
impl Keyboard {
    pub fn new() -> Self {
        Self { pressed_key: 0x0 }
    }

    pub fn is_key_pressed(&self, key: u8) -> bool {
        self.pressed_key == key
    }

    /// Blocks the thread until the key is pressed.
    pub fn wait_for_key(&self) -> u8 {
        while self.pressed_key == 0x0 {
            thread::sleep(Duration::from_millis(10));
        }

        self.pressed_key
    }
}
