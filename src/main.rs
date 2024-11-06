use std::env;

use chip8::Emulator;

mod chip8;
mod cpu;
mod io;
mod keyboard;
mod ram;
mod registers;
mod screen;
mod timer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <program_path>", args[0]);
        std::process::exit(1);
    };

    let program_path: String = args[args.len() - 1].clone();

    let mut emulator = Emulator::new(program_path);
    emulator.start();
}
