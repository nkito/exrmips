use termion::*;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::{thread, time};
use std::io::Read;
use std::sync::Arc;
use std::sync::atomic;

// This function is for non-blocking key inputs.
// 
// Reference:
// https://stackoverflow.com/questions/30012995/how-can-i-read-non-blocking-from-stdin

pub fn spawn_stdin_channel() -> (Receiver<u8>, Arc<atomic::AtomicUsize>) {
    let (tx, rx) = mpsc::channel::<u8>();
    let ctrlc_count : Arc<atomic::AtomicUsize> = Arc::new(atomic::AtomicUsize::new(0));
    let ctrlc_num = Arc::clone(&ctrlc_count);
    thread::spawn(move || {
        let mut stdin_bytes = async_stdin().bytes();
        loop {
            match stdin_bytes.next() {
                Some(Ok(d)) => { 
                    if d == 3 {
                        ctrlc_num.fetch_add(1, atomic::Ordering::Relaxed);
                        //std::process::exit(0);
                    }else{
                        tx.send(d).unwrap();
                    }
                }
                _ => { sleep(10); }
            }
        }
    });
    (rx, ctrlc_count)
}

fn sleep(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}