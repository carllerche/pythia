use {sleep_ms};
use syncbox::{ThreadPool, Run};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::*;

#[test]
pub fn test_one_thread_basic() {
    let tp = ThreadPool::single_thread();
    let (tx, rx) = sync_channel(0);

    tp.run(move || {
        tx.send("hi").unwrap();
    });

    assert_eq!("hi", rx.recv().unwrap());
}

#[test]
pub fn test_two_thread_basic() {
    let tp = ThreadPool::fixed_size(2);
    let (tx, rx) = sync_channel(0);

    for i in (0..2i32) {
        let tx = tx.clone();
        tp.run(move || {
            debug!("send; task={}; msg=hi", i);
            tx.send("hi").unwrap();
            sleep_ms(500);

            debug!("send; task={}; msg=bye", i);
            tx.send("bye").unwrap();
            sleep_ms(500);
        });
    }

    debug!("recv");

    for &msg in ["hi", "hi", "bye", "bye"].iter() {
        assert_eq!(msg, rx.recv().unwrap());
    }
}

#[test]
pub fn test_two_threads_task_queue_up() {
    let tp = ThreadPool::fixed_size(2);
    let (tx, rx) = sync_channel(0);

    for i in (0..4i32) {
        let tx = tx.clone();
        tp.run(move || {
            debug!("send; task={}; msg=hi", i);
            tx.send("hi").unwrap();
            sleep_ms(500);

            debug!("send; task={}; msg=bye", i);
            tx.send("bye").unwrap();
            sleep_ms(500);
        });
    }

    debug!("recv");

    for &msg in ["hi", "hi", "bye", "bye", "hi", "hi", "bye", "bye"].iter() {
        assert_eq!(msg, rx.recv().unwrap());
    }
}

#[test]
pub fn test_thread_pool_is_send() {
    fn check<R: Run<F> + Send, F: FnOnce() + Send + 'static>(_: &R) {
    }

    let tp = ThreadPool::fixed_size(2);
    check(&tp);
    tp.run(|| { assert!(true); });
}

#[test]
pub fn test_thread_pool_drains_queue_before_shutdown() {
    // This is a racy test, so we run it a bunch of times
    'repeat:
    for _ in 0..20 {
        let tp = ThreadPool::single_thread();
        let cnt = Arc::new(AtomicUsize::new(0));

        for _ in 0..20 {
            let cnt = cnt.clone();
            tp.run(move || {
                cnt.fetch_add(1, Ordering::Relaxed);
            });
        }

        // Drop the pool, cleanly shutdown
        drop(tp);

        // Wait up to 1 second
        for _ in 0..20 {
            let i = cnt.load(Ordering::Relaxed);

            if i == 20 {
                break 'repeat;
            }

            sleep_ms(50);
        }

        panic!("failed to run all submitted tasks");
    }
}
