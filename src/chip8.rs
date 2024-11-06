use std::fs;

use crate::cpu::CPU;

pub struct Emulator {
    cpu: CPU,
}
impl Emulator {
    pub fn new(program_path: String) -> Self {
        let program = fs::read(program_path).expect("Failed to read program!");

        let mut cpu = CPU::new();
        cpu.load_rom(&program)
            .expect("Could not load ROM into RAM!");

        Emulator { cpu }
    }

    pub fn start(&mut self) {
        self.cpu.clock();
    }
}
