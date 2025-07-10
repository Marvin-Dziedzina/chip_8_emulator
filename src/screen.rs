use bevy::prelude::*;

const COLLUMNS: usize = 64;
const ROWS: usize = 32;

#[derive(Debug)]
pub struct Screen {
    screen: [u8; 64 * 32],
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            screen: [0u8; COLLUMNS * ROWS],
        }
    }

    pub fn clear(&mut self) {
        self.screen = [0u8; COLLUMNS * ROWS]
    }

    pub fn draw(&mut self) {
        todo!("Draw sprite onto screen.");
        //debug!("Fake Drawing!");
    }
}
