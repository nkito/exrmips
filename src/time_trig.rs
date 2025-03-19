use std::{thread, time};
use std::sync::Arc;
use std::sync::atomic;
use crate::config;

pub fn spawn_time_trigger() -> Arc<atomic::AtomicBool> {
    let time_trigger : Arc<atomic::AtomicBool> = Arc::new(atomic::AtomicBool::new(false));
    let time_flag = Arc::clone(&time_trigger);
    thread::spawn(move || loop {
        thread::sleep(time::Duration::from_micros(config::SYSTEM_TIMER_INTERVAL_IN_USEC as u64));
        time_flag.swap(true, atomic::Ordering::Relaxed);
    });
    time_trigger
}
