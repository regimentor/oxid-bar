use audio::backend::pulse::start_listening;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    logger::init();
    logger::log_info("main", "Application started");

    let terminate = Arc::new(AtomicBool::new(false));
    let terminate_clone = terminate.clone();

    ctrlc::set_handler(move || {
        terminate_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    std::thread::spawn(|| {
        logger::log_info("audio-thread", "Audio thread started");
        start_listening::start_listening().unwrap_or_else(|e| {
            logger::log_error("main", format!("Error starting audio listening: {}", e));
        });
    });

    while !terminate.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    logger::log_info("main", "Application finished");
}
