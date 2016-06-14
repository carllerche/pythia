use syncbox::{ScheduledThreadPool};
use std::sync::mpsc::*;
use std::thread;
use std::time::{Instant, Duration};

#[test]
pub fn test_one_thread_one_task() {
    let tp = ScheduledThreadPool::single_thread();
    let (tx, rx) = channel();

    let start = Instant::now();

    tp.schedule_ms(500, move || {
        tx.send(start.elapsed() > ms(500)).unwrap();
    });

    assert!(rx.recv().unwrap());
}

#[test]
pub fn test_one_thread_two_tasks() {
    let tp = ScheduledThreadPool::single_thread();
    let (tx, rx) = channel();

    let start = Instant::now();

    {
        let tx = tx.clone();
        tp.schedule_ms(500, move || {
            tx.send(("one", start.elapsed() > ms(500))).unwrap();
        });
    }

    {
        let tx = tx.clone();
        tp.schedule_ms(200, move || {
            tx.send(("two", start.elapsed() > ms(200))).unwrap();
        });
    }

    assert_eq!(rx.recv().unwrap(), ("two", true));
    assert_eq!(rx.recv().unwrap(), ("one", true));
}

#[test]
pub fn test_two_threads() {
    let tp = ScheduledThreadPool::fixed_size(2);
    let (tx, rx) = channel();

    let start = Instant::now();

    {
        let tx = tx.clone();
        tp.schedule_ms(500, move || {
            assert!(start.elapsed() > ms(500));
            tx.send("win").unwrap();
        });
    }

    {
        let tx = tx.clone();
        tp.schedule_ms(100, move || {
            assert!(start.elapsed() > ms(100));
            tx.send("start").unwrap();
            thread::sleep(Duration::from_secs(2));
            tx.send("end").unwrap();
        });
    }

    drop(tx);

    let vals: Vec<&'static str> = rx.iter().take(3).collect();
    assert_eq!(vals, &["start", "win", "end"]);
}

fn ms(ms: u64) -> Duration {
    Duration::from_millis(ms)
}
