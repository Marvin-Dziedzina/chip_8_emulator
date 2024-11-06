use std::{
    ops::{Add, Sub},
    thread,
    time::{Duration, Instant},
};

use rand::Rng;

use crate::{
    io::{MemoryError, Read, Write},
    keyboard::Keyboard,
    ram::{Stack, RAM},
    registers::{I, V},
    screen::Screen,
    timer::{DelayTimer, SoundTimer},
};

#[derive(Debug)]
pub struct CPU {
    is_paused: bool,

    // Clock speed in Hz
    clock_speed: f64,
    program_counter: u16,
    ram: RAM,
    stack: Stack,
    sound_timer: SoundTimer,
    delay_timer: DelayTimer,
    v: V,
    i: I,

    screen: Screen,
    keyboard: Keyboard,
}
impl CPU {
    pub fn new() -> Self {
        CPU {
            is_paused: false,

            clock_speed: 500.0,
            program_counter: 0x200,
            ram: RAM::new(),
            stack: Stack::new(),
            sound_timer: SoundTimer::new(),
            delay_timer: DelayTimer::new(),
            v: V::new(),
            i: I::new(),

            screen: Screen::new(),
            keyboard: Keyboard::new(),
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), MemoryError> {
        self.ram.write_buf(0x200, data)
    }

    pub fn cycle(&mut self) {
        let opcode = (self.ram.read(self.program_counter).unwrap() as u16) << 8
            | self.ram.read(self.program_counter + 1).unwrap() as u16;

        self.execute_instruction(opcode);
    }

    fn execute_instruction(&mut self, opcode: u16) {
        // Increment the program counter by 2 because one instruction is 2 bytes long (u16).
        self.increment_program_counter();

        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;

        // match instructions
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.screen.clear(),
                0x00EE => {
                    self.program_counter = self.stack.pop().expect("Could not pop off of stack!")
                }
                x => panic!("Invalid instruction received! {}", x),
            },
            0x1000 => self.program_counter = opcode & 0xFFF,
            0x2000 => {
                let nnn = opcode & 0xFFF;
                self.stack
                    .push(nnn)
                    .expect("Could not push on to the stack!");
                self.program_counter = nnn;
            }
            0x3000 => {
                let vx = self.v.read(x).expect(&format!("Could not read V({})!", x));
                let kk = (opcode & 0xFF) as u8;
                if vx == kk {
                    self.increment_program_counter();
                };
            }
            0x4000 => {
                let vx = self.v.read(x).expect(&format!("Could not read V({})!", x));
                let kk = (opcode & 0xFF) as u8;
                if vx != kk {
                    self.increment_program_counter();
                };
            }
            0x5000 => {
                let vx = self.v.read(x).expect(&format!("Could not read V({})!", x));
                let vy = self.v.read(y).expect(&format!("Could not read V({})!", x));

                if vx == vy {
                    self.increment_program_counter();
                };
            }
            0x6000 => {
                let kk = (opcode & 0xFF) as u8;
                self.v
                    .write(x, kk)
                    .expect(&format!("Could not write {} to V({})", kk, x));
            }
            0x7000 => {
                let vx = self.v.read(x).expect(&format!("Could not read V({})!", x));
                let kk = (opcode & 0xFF) as u8;
                self.v.write(x, vx + kk).expect(&format!(
                    "Could not write {} to V({})!",
                    vx + kk,
                    x
                ));
            }
            0x8000 => match opcode & 0xF {
                0x0 => self
                    .v
                    .write(
                        x,
                        self.v.read(y).expect(&format!("Could not read V({})!", y)),
                    )
                    .expect(&format!("Could not write V({}) to V({})!", y, x)),
                0x1 => {
                    let vx = self.v.read(x).expect(&format!("Could not read V({})", x));
                    let vy = self.v.read(y).expect(&format!("Could not read V({})", y));
                    self.v.write(x, vx | vy).expect(&format!(
                        "Could not write {} to V({})",
                        vx | vy,
                        x
                    ));
                }
                0x2 => {
                    let vx = self.v.read(x).expect(&format!("Could not read V({})", x));
                    let vy = self.v.read(y).expect(&format!("Could not read V({})", y));
                    self.v.write(x, vx & vy).expect(&format!(
                        "Could not write {} to V({})",
                        vx & vy,
                        x
                    ));
                }
                0x3 => {
                    let vx = self.v.read(x).expect(&format!("Could not read V({})", x));
                    let vy = self.v.read(y).expect(&format!("Could not read V({})", y));
                    self.v.write(x, vx ^ vy).expect(&format!(
                        "Could not write {} to V({})",
                        vx ^ vy,
                        x
                    ));
                }
                0x4 => {
                    let vx = self.v.read(x).expect(&format!("Could not read V({})", x));
                    let vy = self.v.read(y).expect(&format!("Could not read V({})", y));

                    let result = (vx as u16).add(vy as u16);

                    let carry = if result > 0xFF { 1 } else { 0 };

                    // Set carry
                    self.v
                        .write(0xF, carry)
                        .expect(&format!("Could not write carry to V({})!", 0xF));

                    self.v.write(x, result as u8).expect(&format!(
                        "Could not write sum of {} and {} to V({})!",
                        vx, vy, x
                    ));
                }
                0x5 => {
                    let vx = self.v.read(x).expect(&format!("Could not read V({})", x));
                    let vy = self.v.read(y).expect(&format!("Could not read V({})", y));

                    let result = vx.sub(vy);

                    let carry = if vx > vy { 1 } else { 0 };

                    // Set carry
                    self.v
                        .write(0xF, carry)
                        .expect(&format!("Could not write carry to V({})!", 0xF));

                    self.v.write(x, result).expect(&format!(
                        "Could not write sum of {} and {} to V({})!",
                        vx, vy, x
                    ));
                }
                0x6 => {
                    let vx = self.v.read(x).expect(&format!("Could not read V({})", x)) & 0x1;

                    self.v
                        .write(0xF, vx)
                        .expect(&format!("Could not write {} to V({})", vx, x));
                }
                0x7 => {
                    let vx = self.v.read(x).expect(&format!("Could not read V({})", x));
                    let vy = self.v.read(y).expect(&format!("Could not read V({})", y));

                    let carry = if vy > vx { 1 } else { 0 };
                    self.v
                        .write(0xF, carry)
                        .expect(&format!("Could not write {} to V({})!", carry, 0xF));

                    let result = (vy as u16).sub(vx as u16);
                    self.v
                        .write(x, result as u8)
                        .expect(&format!("Could not write {} to V({})!", result, x));
                }
                0xE => {
                    let vx = self.v.read(x).expect(&format!("Could not read V({})!", x));
                    self.v.write(0xF, vx & 0x80).expect(&format!(
                        "Could not write {} to V({})!",
                        vx & 0x80,
                        x
                    ));
                    self.v.write(x, vx << 1).expect(&format!(
                        "Could not write {} to V({})!",
                        vx << 1,
                        x
                    ));
                }
                x => panic!("Invalid instruction received! {}", x),
            },
            0x9000 => {
                let vx = self.v.read(x).expect(&format!("Could not read V({})", x));
                let vy = self.v.read(y).expect(&format!("Could not read V({})", y));

                if vx != vy {
                    self.increment_program_counter();
                };
            }
            0xA000 => {
                self.i.write(opcode & 0xFFF);
            }
            0xB000 => {
                self.program_counter = (opcode & 0xFFF)
                    + self
                        .v
                        .read(0x0)
                        .expect(&format!("Could not read V({})!", 0x0))
                        as u16;
            }
            0xC000 => {
                let kk = (opcode & 0xFF) as u8;
                let rand_num: u8 = rand::thread_rng().gen::<u8>();

                self.v.write(x, rand_num & kk).expect(&format!(
                    "Could not write {} to V({})!",
                    rand_num & kk,
                    x
                ));
            }
            0xD000 => {
                const width: u8 = 8;
                let height = (opcode & 0xF) as u8;

                todo!("Draw sprite!");
            }
            0xE000 => {
                match opcode & 0xFF {
                    0x9E => {
                        if self.keyboard.is_key_pressed(
                            self.v.read(x).expect(&format!("Could not read V({})!", x)),
                        ) {
                            self.increment_program_counter();
                        };
                    }
                    0xA1 => {
                        if !self.keyboard.is_key_pressed(
                            self.v.read(x).expect(&format!("Could not read V({})!", x)),
                        ) {
                            self.increment_program_counter();
                        };
                    }
                    x => panic!("Invalid instruction received! {}", x),
                }
            }
            0xF000 => match opcode & 0xFF {
                0x0F => self.v.write(x, self.delay_timer.read()).expect(&format!(
                    "Could not write {} to V({})!",
                    self.delay_timer.read(),
                    x
                )),
                0x0A => {
                    self.is_paused = true;

                    let key = self.keyboard.wait_for_key();
                    self.v
                        .write(x, key)
                        .expect(&format!("Could not write {} to V({})!", key, x));
                }
                0x15 => {
                    self.delay_timer
                        .write(self.v.read(x).expect(&format!("Could not read V({})!", x)));
                }
                0x18 => {
                    self.sound_timer
                        .write(self.v.read(x).expect(&format!("Could not read V({})!", x)));
                }
                0x1E => {
                    self.i.write(
                        self.i.read()
                            + self.v.read(x).expect(&format!("Could not read V({})!", x)) as u16,
                    );
                }
                0x29 => {
                    self.i.write(
                        self.v.read(x).expect(&format!("Could not read V({})!", x)) as u16 * 5,
                    );
                }
                0x33 => {
                    self.ram
                        .write(
                            self.i.read(),
                            // Get hundrets digit.
                            self.v.read(x).expect(&format!("Could not read V({})!", x)) / 100,
                        )
                        .expect(&format!("Could not write RAM({})!", x));

                    self.ram
                        .write(
                            self.i
                                .read()
                                .checked_add(1)
                                .expect(&format!("Could not add 1 to I {}!", self.i.read())),
                            // Get value of the tens digit.
                            (self.v.read(x).expect(&format!("Could not read V({})!", x)) % 100)
                                / 10,
                        )
                        .expect(&format!("Could not write RAM({})!", x));

                    self.ram
                        .write(
                            self.i
                                .read()
                                .checked_add(2)
                                .expect(&format!("Could not add 2 to I {}!", self.i.read())),
                            // Get value of the ones digit
                            self.v.read(x).expect(&format!("Could not read V({})!", x)) % 10,
                        )
                        .expect(&format!("Could not write RAM({})!", x));
                }
                0x55 => self
                    .ram
                    .write_buf(
                        self.i.read(),
                        self.v
                            .read_range(0, x)
                            .expect(&format!("Could not read range V(0, {})!", x)),
                    )
                    .expect(&format!(
                        "Could not write V(0, {}) in RAM({}, {})!",
                        x,
                        self.i.read(),
                        self.i.read() + x as u16
                    )),
                0x65 => self
                    .v
                    .write_buf(
                        0,
                        self.ram
                            .read_range(self.i.read(), x as u16)
                            .expect(&format!(
                                "Could not read range from RAM({}, {})!",
                                self.i.read(),
                                x
                            )),
                    )
                    .expect(&format!(
                        "Could not write RAM({}, {}) to V(0)!",
                        self.i.read(),
                        x
                    )),
                x => panic!("Invalid instruction received! {}", x),
            },
            x => panic!("Invalid instruction received! {}", x),
        };
    }

    pub fn clock(&mut self) {
        let clock_duration = Duration::from_secs_f64(1. / self.clock_speed);

        loop {
            let start = Instant::now();

            if !self.is_paused {
                self.cycle();
            };

            if let Some(waiting_duration) = clock_duration.checked_sub(start.elapsed()) {
                thread::sleep(waiting_duration);
            };
        }
    }

    fn increment_program_counter(&mut self) {
        self.program_counter += 2;
    }
}
