use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct SoundTimer {
    value: Arc<Mutex<u8>>,
}
impl SoundTimer {
    pub fn new() -> Self {
        Self {
            value: Arc::new(Mutex::new(0)),
        }
    }

    pub fn write(&mut self, value: u8) {
        let value_c = self.value.clone();

        let mut value_lock = self.value.lock().unwrap_or_else(|p| p.into_inner());
        *value_lock = value;

        if *value_lock > 0 {
            thread::spawn(move || {
                decrement60hz(value_c);
            });
        }
    }

    pub fn read(&self) -> u8 {
        let value_lock = self.value.lock().unwrap_or_else(|p| p.into_inner());
        *value_lock
    }
}

#[derive(Debug)]
pub struct DelayTimer {
    value: Arc<Mutex<u8>>,
}
impl DelayTimer {
    pub fn new() -> Self {
        Self {
            value: Arc::new(Mutex::new(0)),
        }
    }

    pub fn write(&mut self, value: u8) {
        let value_c = self.value.clone();

        let mut value_lock = self.value.lock().unwrap_or_else(|p| p.into_inner());
        *value_lock = value;

        if *value_lock > 0 {
            thread::spawn(move || {
                decrement60hz(value_c);
            });
        };
    }

    pub fn read(&self) -> u8 {
        let value_lock = self.value.lock().unwrap_or_else(|p| p.into_inner());
        *value_lock
    }
}

fn decrement60hz(value: Arc<Mutex<u8>>) {
    let target_duration = Duration::from_secs_f64(1. / 60.); // 60Hz

    loop {
        let start = Instant::now();

        {
            let mut value_lock = value.lock().unwrap_or_else(|p| p.into_inner());

            if *value_lock > 0 {
                *value_lock -= 1;
            } else {
                break;
            };
        }

        if let Some(sleep_duration) = target_duration.checked_sub(start.elapsed()) {
            thread::sleep(sleep_duration);
        };
    }
}