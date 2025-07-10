use bevy::{prelude::*, window::PresentMode};

use cpu::CPUPlugin;

mod cpu;
mod io;
mod keyboard;
mod ram;
mod registers;
mod screen;
mod timer;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::Fifo,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(CPUPlugin)
        .run()
}
