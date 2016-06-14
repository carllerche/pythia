extern crate syncbox;

#[macro_use]
extern crate log;
extern crate env_logger;

mod test_delay_queue;
mod test_linked_queue;
mod test_scheduled_pool;
mod test_thread_pool;

fn spawn<F: FnOnce() + Send + 'static>(f: F) {
    use std::thread;
    thread::spawn(f);
}

fn sleep_ms(ms: u64) {
    use std::thread;
    use std::time::{Duration, Instant};

    let start = Instant::now();
    let target = start + Duration::from_millis(ms);

    loop {
        let now = Instant::now();

        if now > target {
            return;
        }

        thread::park_timeout(target - now);
    }
}
