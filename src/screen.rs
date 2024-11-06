#[derive(Debug)]
pub struct Screen {
    screen: [u8; 64 * 32],
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            screen: [0u8; 64 * 32],
        }
    }

    pub fn clear(&mut self) {
        self.screen = [0u8; 64 * 32]
    }

    pub fn render(&mut self) -> Result<(), ()> {
        todo!("Add screen render capabilities.")
    }
}
