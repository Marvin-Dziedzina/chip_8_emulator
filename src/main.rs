use std::{env, fs};

use cpu::CPU;
use log::error;

mod cpu;
mod io;
mod keyboard;
mod ram;
mod registers;
mod screen;
mod timer;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <program_path>", args[0]);
        error!("No arguments given!");
        std::process::exit(1);
    };

    let program_path: String = args[args.len() - 1].clone();

    let program = fs::read(program_path).expect("Failed to read program!");

    let mut cpu = CPU::new();
    cpu.load_rom(&program)
        .expect("Could not load ROM into RAM!");
    cpu.clock();
}
