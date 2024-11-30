use std::{
    ops::{Add, Sub},
    thread,
    time::{Duration, Instant},
};

use log::{info, trace};
use rand::Rng;

use crate::{
    io::{MemoryError, Read, Write},
    keyboard::Keyboard,
    ram::{Stack, RAM},
    registers::{I, V},
    screen::Screen,
    timer::{DelayTimer, SoundTimer},
};

const SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

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
        let mut ram = RAM::new();
        ram.write_buf(0, &SPRITES)
            .expect("Could not load SPRITES into RAM!");

        trace!("Loaded sprites into RAM.");

        info!("Creating new CPU instance.");

        CPU {
            is_paused: false,

            clock_speed: 500.0,
            program_counter: 0x200,
            ram,
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
        info!("Loading ROM.");
        self.ram.write_buf(0x200, data)
    }

    fn cycle(&mut self) {
        trace!("--- New Cycle ---");
        trace!("Program Counter: {}", self.program_counter);

        let opcode = (self.ram.read(self.program_counter).unwrap() as u16) << 8
            | self.ram.read(self.program_counter + 1).unwrap() as u16;

        trace!("OPCODE: {}", opcode);

        self.execute_instruction(opcode);

        trace!("End of Cycle");
    }

    fn execute_instruction(&mut self, opcode: u16) {
        // Increment the program counter by 2 because one instruction is 2 bytes long (u16).
        self.increment_program_counter();

        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;

        // match instructions
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => {
                    trace!("Clearing screen.");
                    self.screen.clear()
                }
                0x00EE => {
                    self.program_counter = self.stack.pop().expect("Could not pop off of stack!");
                    trace!(
                        "Return from a subroutine. New program counter: {}",
                        self.program_counter
                    );
                }
                _ => {
                    // Instruction 0nnn

                    let nnn = opcode & 0xFFF;

                    trace!("Set ProgramCounter to {}", nnn);

                    self.program_counter = nnn;
                }
            },
            0x1000 => {
                self.program_counter = opcode & 0xFFF;
                trace!("Jump to {}", self.program_counter);
            }
            0x2000 => {
                self.stack.push(self.program_counter).expect(&format!(
                    "Could not push ProgramCounter({}) on to the stack!",
                    self.program_counter
                ));

                let nnn = opcode & 0xFFF;
                self.program_counter = nnn;
                trace!("Call subroutine at {}", nnn);
            }
            0x3000 => {
                trace!("Skip next instruction if V({}) == KK.", x);
                let vx = self.v.read(x).expect(&format!("Could not read V({})!", x));
                let kk = (opcode & 0xFF) as u8;

                if vx == kk {
                    trace!("Skipping next instruction.");
                    self.increment_program_counter();
                };
            }
            0x4000 => {
                trace!("Skip next instruction if V({}) != KK.", x);
                let vx = self.v.read(x).expect(&format!("Could not read V({})!", x));
                let kk = (opcode & 0xFF) as u8;

                if vx != kk {
                    trace!("Skipping next instruction.");
                    self.increment_program_counter();
                };
            }
            0x5000 => {
                trace!("Skip next instruction if V({}) == V({}).", x, y);
                let vx = self.v.read(x).expect(&format!("Could not read V({})!", x));
                let vy = self.v.read(y).expect(&format!("Could not read V({})!", x));

                if vx == vy {
                    trace!("Skipping instruction.");
                    self.increment_program_counter();
                };
            }
            0x6000 => {
                let kk = (opcode & 0xFF) as u8;
                trace!("Setting V({}) to {}", x, kk);
                self.v
                    .write(x, kk)
                    .expect(&format!("Could not write {} to V({})", kk, x));
            }
            0x7000 => {
                let vx = self.v.read(x).expect(&format!("Could not read V({})!", x));
                let kk = (opcode & 0xFF) as u8;
                trace!("Set V({}) to {} + {}", x, vx, kk);
                self.v.write(x, vx.wrapping_add(kk)).expect(&format!(
                    "Could not write {} to V({})!",
                    vx as u16 + kk as u16,
                    x
                ));
            }
            0x8000 => match opcode & 0xF {
                0x0 => {
                    trace!("Set V({}) to V({})", x, y);
                    self.v
                        .write(
                            x,
                            self.v.read(y).expect(&format!("Could not read V({})!", y)),
                        )
                        .expect(&format!("Could not write V({}) to V({})!", y, x))
                }
                0x1 => {
                    trace!("Set V({}) to V({}) | V({})", x, x, y);
                    let vx = self.v.read(x).expect(&format!("Could not read V({})", x));
                    let vy = self.v.read(y).expect(&format!("Could not read V({})", y));
                    self.v.write(x, vx | vy).expect(&format!(
                        "Could not write {} to V({})",
                        vx | vy,
                        x
                    ));
                }
                0x2 => {
                    trace!("Set V({}) to V({}) & V({})", x, x, y);
                    let vx = self.v.read(x).expect(&format!("Could not read V({})", x));
                    let vy = self.v.read(y).expect(&format!("Could not read V({})", y));
                    self.v.write(x, vx & vy).expect(&format!(
                        "Could not write {} to V({})",
                        vx & vy,
                        x
                    ));
                }
                0x3 => {
                    trace!("Set V({}) to V({}) ^ V({})", x, x, y);
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

                    let result = vx.wrapping_add(vy);

                    let carry = if vx as u16 + vy as u16 > u8::MAX as u16 {
                        1
                    } else {
                        0
                    };

                    trace!(
                        "Set V({}) = V({}) + V({}), set V(0xF) = Carry {}",
                        x,
                        x,
                        y,
                        carry
                    );

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

                    let borrow = if vx >= vy { 1 } else { 0 };

                    trace!(
                        "Set V({}) = V({}) {} - V({}) {}, set V(0xF) = Borrow {}",
                        x,
                        x,
                        vx,
                        y,
                        vy,
                        borrow
                    );

                    let result = vx.wrapping_sub(vy);

                    // Set carry
                    self.v
                        .write(0xF, borrow)
                        .expect(&format!("Could not write carry to V({})!", 0xF));

                    self.v.write(x, result as u8).expect(&format!(
                        "Could not write sum of {} and {} to V({})!",
                        vx, vy, x
                    ));
                }
                0x6 => {
                    let vx = self.v.read(x).expect(&format!("Could not read V({})", x)) & 0x1;

                    trace!("Set V({}) = V({}) SHR 1", x, x);

                    self.v
                        .write(0xF, vx)
                        .expect(&format!("Could not write {} to V({})", vx, x));
                    self.v
                        .write(
                            x,
                            self.v.read(x).expect(&format!("Could not read V({})!", x)) >> 1,
                        )
                        .expect(&format!("Could not write to V({})", x));
                }
                0x7 => {
                    let vx = self.v.read(x).expect(&format!("Could not read V({})", x));
                    let vy = self.v.read(y).expect(&format!("Could not read V({})", y));

                    let borrow = if vy > vx { 1 } else { 0 };

                    trace!(
                        "Set V({}) = V({}) - V({}), set V(0xF) = Borrow {}",
                        x,
                        x,
                        y,
                        borrow
                    );

                    self.v
                        .write(0xF, borrow)
                        .expect(&format!("Could not write {} to V({})!", borrow, 0xF));

                    let result = vy.wrapping_sub(vx);
                    self.v
                        .write(x, result as u8)
                        .expect(&format!("Could not write {} to V({})!", result, x));
                }
                0xE => {
                    let vx = self.v.read(x).expect(&format!("Could not read V({})!", x));

                    trace!("Set V({}) = V({}) SHL 1", x, x);

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

                trace!("Skip next instruction if V({}) != V({})", x, y);

                if vx != vy {
                    trace!("Skipping next instruction");
                    self.increment_program_counter();
                };
            }
            0xA000 => {
                let nnn = opcode & 0xFFF;
                trace!("Set I = {}", nnn);
                self.i.write(nnn);
            }
            0xB000 => {
                let nnn = opcode & 0xFFF;
                let v0 = self
                    .v
                    .read(0x0)
                    .expect(&format!("Could not read V({})!", 0x0));
                self.program_counter = nnn + v0 as u16;

                trace!("Jump to location {} + {} = {}", nnn, v0, nnn + v0 as u16);
            }
            0xC000 => {
                let kk = (opcode & 0xFF) as u8;
                let rand_num: u8 = rand::thread_rng().gen::<u8>();

                trace!("Set V({}) = RAND BYTE {} & {}", x, rand_num, kk);

                self.v.write(x, rand_num & kk).expect(&format!(
                    "Could not write {} to V({})!",
                    rand_num & kk,
                    x
                ));
            }
            0xD000 => {
                trace!("Display n-byte sprite starting at memory location I at (V({}), V({})), set V(0xF) = Collision {}", x, y, -1);
                self.screen.draw();
            }
            0xE000 => {
                match opcode & 0xFF {
                    0x9E => {
                        trace!(
                            "Skip next instruction if key with the value of V({}) is pressed",
                            x
                        );
                        if self.keyboard.is_key_pressed(
                            self.v.read(x).expect(&format!("Could not read V({})!", x)),
                        ) {
                            trace!("Skipping next instruction");
                            self.increment_program_counter();
                        };
                    }
                    0xA1 => {
                        trace!(
                            "Skip next instruction if key with the value of V({}) is not pressed",
                            x
                        );
                        if !self.keyboard.is_key_pressed(
                            self.v.read(x).expect(&format!("Could not read V({})!", x)),
                        ) {
                            trace!("Skipping next instruction");
                            self.increment_program_counter();
                        };
                    }
                    x => panic!("Invalid instruction received! {}", x),
                }
            }
            0xF000 => {
                match opcode & 0xFF {
                    0x07 => {
                        let delaytimer_value = self.delay_timer.read();
                        trace!("Write delaytimer {} into V({})", delaytimer_value, x);

                        self.v.write(x, delaytimer_value).expect(&format!(
                            "Could not write delaytimer {} into v({})!",
                            delaytimer_value, x
                        ));
                    }
                    0x0F => {
                        let delay_timer = self.delay_timer.read();
                        trace!("Set V({}) = Delay Timer {}", x, delay_timer);
                        self.v.write(x, self.delay_timer.read()).expect(&format!(
                            "Could not write {} to V({})!",
                            self.delay_timer.read(),
                            x
                        ))
                    }
                    0x0A => {
                        self.is_paused = true;

                        trace!("Wait for a key press");

                        let key = self.keyboard.wait_for_key();
                        self.v
                            .write(x, key)
                            .expect(&format!("Could not write {} to V({})!", key, x));

                        trace!(
                            "Key {} pressed, stored the value of the key in V({})",
                            key,
                            x
                        );

                        self.is_paused = false;
                    }
                    0x15 => {
                        trace!("Set delay timer = V({})", x);
                        self.delay_timer
                            .write(self.v.read(x).expect(&format!("Could not read V({})!", x)));
                    }
                    0x18 => {
                        trace!("Set sound timer = V({})", x);
                        self.sound_timer
                            .write(self.v.read(x).expect(&format!("Could not read V({})!", x)));
                    }
                    0x1E => {
                        trace!("Set I = I{} + V({})", self.i.read(), x);
                        self.i.write(self.i.read().wrapping_add(
                            self.v.read(x).expect(&format!("Could not read V({})!", x)) as u16,
                        ));
                    }
                    0x29 => {
                        trace!("Set I = location of sprite for digit V({})", x);
                        self.i.write(
                            self.v.read(x).expect(&format!("Could not read V({})!", x)) as u16 * 5,
                        );
                    }
                    0x33 => {
                        let i = self.i.read();
                        trace!("Store BCD representation of V({}) in memory locations I{}, I{}+1, and I{}+2", x, i, i, i);

                        self.ram
                            .write(
                                i,
                                // Get hundrets digit.
                                self.v.read(x).expect(&format!("Could not read V({})!", x)) / 100,
                            )
                            .expect(&format!("Could not write RAM({})!", x));

                        self.ram
                            .write(
                                i.checked_add(1)
                                    .expect(&format!("Could not add 1 to I {}!", i)),
                                // Get value of the tens digit.
                                (self.v.read(x).expect(&format!("Could not read V({})!", x)) % 100)
                                    / 10,
                            )
                            .expect(&format!("Could not write RAM({})!", x));

                        self.ram
                            .write(
                                i.checked_add(2)
                                    .expect(&format!("Could not add 2 to I {}!", i)),
                                // Get value of the ones digit
                                self.v.read(x).expect(&format!("Could not read V({})!", x)) % 10,
                            )
                            .expect(&format!("Could not write RAM({})!", x));
                    }
                    0x55 => {
                        let i = self.i.read();
                        trace!(
                            "Store registers V(0) through V({}) in memory starting at location I{}",
                            x,
                            i
                        );
                        self.ram
                            .write_buf(
                                i,
                                self.v
                                    .read_range(0, x)
                                    .expect(&format!("Could not read range V(0, {})!", x)),
                            )
                            .expect(&format!(
                                "Could not write V(0, {}) in RAM({}, {})!",
                                x,
                                i,
                                i + x as u16
                            ))
                    }
                    0x65 => {
                        let i = self.i.read();
                        trace!("Read registers V(0) through V({}) from memory starting at location I{}", x, i);
                        self.v
                            .write_buf(
                                0,
                                self.ram.read_range(i, x as u16).expect(&format!(
                                    "Could not read range from RAM({}, {})!",
                                    i, x
                                )),
                            )
                            .expect(&format!("Could not write RAM({}, {}) to V(0)!", i, x))
                    }
                    x => panic!("Invalid instruction received! {}", x),
                }
            }
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
                trace!("Waiting {} ns", waiting_duration.as_nanos());
                thread::sleep(waiting_duration);
            };
        }
    }

    fn increment_program_counter(&mut self) {
        self.program_counter += 2;
        trace!("Incremented Program Counter.");
    }
}
